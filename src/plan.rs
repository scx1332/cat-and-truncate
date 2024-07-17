use crate::ops::{copy_chunk, output_chunk, truncate_file};
use anyhow::bail;

pub struct ChunkPlan {
    chunk_size: u64,
    file_size: u64,
    start_chunks: u64,
    middle_left_size: u64,
    middle_right_size: u64,
}

pub fn plan_chunks(chunk_size: u64, file_size: u64) -> anyhow::Result<ChunkPlan> {
    if file_size == 0 {
        bail!("File size is 0");
    }

    let start_chunks = file_size / 2 / chunk_size;

    let middle_size = file_size - (2 * start_chunks * chunk_size);

    let middle_right_size = middle_size / 2;
    let middle_left_size = middle_size - middle_right_size;
    Ok(ChunkPlan {
        chunk_size,
        file_size,
        start_chunks,
        middle_left_size,
        middle_right_size,
    })
}

pub struct Operation {
    pub chunk_no: u64,
    pub src_chunk: Option<(u64, u64)>,
    pub data_chunk: (u64, u64),
    pub truncate_to: u64,
    pub is_middle: bool,
}

pub fn commit_plan(
    file_path: Option<&str>,
    operations: &[Operation],
    dry_run: bool,
) -> anyhow::Result<()> {
    let mut step_no = 0;
    for op in operations {
        let middle_msg = if op.is_middle { "(middle) " } else { "" };
        log::info!(
            "{} - {}Output chunk {} - {}-{}",
            step_no,
            middle_msg,
            op.chunk_no,
            op.data_chunk.0,
            op.data_chunk.1
        );
        if !dry_run {
            output_chunk(file_path.expect("file path expected"), op.data_chunk).unwrap();
        }
        step_no += 1;
        if let Some((src_start, src_end)) = op.src_chunk {
            log::info!(
                "{} - Copy {} bytes from {}-{} to {}-{}",
                step_no,
                src_end - src_start,
                src_start,
                src_end,
                op.data_chunk.0,
                op.data_chunk.1
            );
            if !dry_run {
                let data_chunk = if op.is_middle && (src_end - src_start + 1 == op.data_chunk.1 - op.data_chunk.0) {
                    (op.data_chunk.0, op.data_chunk.1 - 1)
                } else {
                    op.data_chunk
                };
                copy_chunk(
                    file_path.expect("file path expected"),
                    (src_start, src_end),
                    data_chunk,
                )
                .unwrap();
            }
        }
        step_no += 1;
        log::info!("{} - Truncate file to {} bytes", step_no, op.truncate_to);
        if !dry_run {
            truncate_file(file_path.expect("file path expected"), op.truncate_to).unwrap();
        }
        step_no += 1;
    }
    Ok(())
}

pub fn plan_into_realization(plan: ChunkPlan) -> anyhow::Result<Vec<Operation>> {
    let mut operations = Vec::new();
    let mut operation_no = 0;
    let operation_limit = 1000000;
    log::info!(
        "Realizing plan for file size {} and chunk size {}",
        plan.file_size,
        plan.chunk_size
    );
    for i in 0..plan.start_chunks {
        let dst_chunk_start = i * plan.chunk_size;
        let dst_chunk_end = dst_chunk_start + plan.chunk_size;
        let src_chunk_start = plan.file_size - ((i + 1) * plan.chunk_size);
        let src_chunk_end = src_chunk_start + plan.chunk_size;

        operations.push(Operation {
            chunk_no: operation_no,
            src_chunk: Some((src_chunk_start, src_chunk_end)),
            data_chunk: (dst_chunk_start, dst_chunk_end),
            truncate_to: plan.file_size - ((i + 1) * plan.chunk_size),
            is_middle: false,
        });
        operation_no += 1;
        if operation_no > operation_limit {
            bail!("Operation limit reached {}", operation_limit);
        }
    }

    if plan.middle_left_size > 0 {
        let dst_chunk_start = plan.start_chunks * plan.chunk_size;
        let dst_chunk_end = dst_chunk_start + plan.middle_left_size;
        let src_chunk_start = dst_chunk_start + plan.middle_left_size;
        let src_chunk_end = src_chunk_start + plan.middle_right_size;

        let src_chunk = if plan.middle_right_size > 0 {
            Some((src_chunk_start, src_chunk_end))
        } else {
            None
        };
        operations.push(Operation {
            chunk_no: operation_no,
            src_chunk,
            data_chunk: (dst_chunk_start, dst_chunk_end),
            truncate_to: plan.chunk_size * plan.start_chunks + plan.middle_right_size,
            is_middle: true,
        });
        operation_no += 1
    }
    if plan.middle_right_size > 0 {
        let dst_chunk_start = plan.start_chunks * plan.chunk_size;
        let dst_chunk_end = dst_chunk_start + plan.middle_right_size;

        operations.push(Operation {
            chunk_no: operation_no,
            src_chunk: None,
            data_chunk: (dst_chunk_start, dst_chunk_end),
            truncate_to: plan.chunk_size * plan.start_chunks,
            is_middle: true,
        });
        operation_no += 1
    }
    for i in 0..plan.start_chunks {
        let chunk_no = plan.start_chunks - i - 1;
        let dst_chunk_start = chunk_no * plan.chunk_size;
        let dst_chunk_end = dst_chunk_start + plan.chunk_size;

        operations.push(Operation {
            chunk_no: operation_no,
            src_chunk: None,
            data_chunk: (dst_chunk_start, dst_chunk_end),
            truncate_to: plan.chunk_size * chunk_no,
            is_middle: false,
        });
        operation_no += 1;
        if operation_no > operation_limit {
            bail!("Operation limit reached {}", operation_limit);
        }
    }
    Ok(operations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_chunks_1() {
        let chunk_size = 10;
        let file_size = 100;
        let plan = plan_chunks(chunk_size, file_size).unwrap();

        assert_eq!(plan.start_chunks, 5);
        assert_eq!(plan.middle_left_size, 0);
        assert_eq!(plan.middle_right_size, 0);
        assert_eq!(plan.chunk_size, 10);
        assert_eq!(plan.file_size, 100);
    }

    #[test]
    fn test_plan_chunks_2() {
        let chunk_size = 1;
        let file_size = 1;
        let plan = plan_chunks(chunk_size, file_size).unwrap();

        assert_eq!(plan.start_chunks, 0);
        assert_eq!(plan.middle_left_size, 1);
        assert_eq!(plan.middle_right_size, 0);
        assert_eq!(plan.chunk_size, 1);
        assert_eq!(plan.file_size, 1);
    }

    #[test]
    fn test_plan_chunks_3() {
        let chunk_size = 1;
        let file_size = 2;
        let plan = plan_chunks(chunk_size, file_size).unwrap();

        assert_eq!(plan.start_chunks, 1);
        assert_eq!(plan.middle_left_size, 0);
        assert_eq!(plan.middle_right_size, 0);
    }

    #[test]
    fn test_plan_chunks_4() {
        let chunk_size = 1;
        let file_size = 11;
        let plan = plan_chunks(chunk_size, file_size).unwrap();

        assert_eq!(plan.start_chunks, 5);
        assert_eq!(plan.middle_left_size, 1);
        assert_eq!(plan.middle_right_size, 0);
    }

    #[test]
    fn test_plan_chunks_5() {
        let chunk_size = 5;
        let file_size = 19;
        let plan = plan_chunks(chunk_size, file_size).unwrap();

        assert_eq!(plan.start_chunks, 1);
        assert_eq!(plan.middle_left_size, 5);
        assert_eq!(plan.middle_right_size, 4);
    }

    #[test]
    fn test_plan_chunks_6() {
        let chunk_size = 100;
        let file_size = 1001;
        let plan = plan_chunks(chunk_size, file_size).unwrap();

        assert_eq!(plan.start_chunks, 5);
        assert_eq!(plan.middle_left_size, 1);
        assert_eq!(plan.middle_right_size, 0);
    }
}

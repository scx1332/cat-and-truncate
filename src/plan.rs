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

pub fn realize_plan(plan: ChunkPlan) -> anyhow::Result<()> {
    for i in 0..plan.start_chunks {
        println!("Output chunk {}", i);
        println!("Copy {} bytes", plan.chunk_size);
        println!(
            "Truncate file to {} bytes",
            plan.file_size - ((i + 1) * plan.chunk_size)
        );
    }

    if plan.middle_left_size > 0 {
        println!("Output middle left chunk {}", plan.middle_left_size);
        println!("Copy {} bytes", plan.middle_right_size);
        println!(
            "Truncate file to {} bytes",
            plan.chunk_size * plan.start_chunks + plan.middle_right_size
        );
    }
    if plan.middle_right_size > 0 {
        println!("Output middle right chunk {}", plan.middle_right_size);
        println!(
            "Truncate file to {} bytes",
            plan.chunk_size * plan.start_chunks
        );
    }
    for i in 0..plan.start_chunks {
        let chunk_no = plan.start_chunks - i - 1;
        println!("Output chunk {}", chunk_no);
        println!("Copy {} bytes", plan.chunk_size);
        println!("Truncate file to {} bytes", plan.chunk_size * chunk_no);
    }
    Ok(())
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
        let chunk_size = 2;
        let file_size = 11;
        let plan = plan_chunks(chunk_size, file_size).unwrap();

        assert_eq!(plan.start_chunks, 2);
        assert_eq!(plan.middle_left_size, 2);
        assert_eq!(plan.middle_right_size, 1);
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

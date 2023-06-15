use rand::prelude::SliceRandom;
use rand::Rng;
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

pub struct UserMonkey {
    target: String,
    files: Vec<PathBuf>,
    amount_of_files: u32,
    seconds: u32,
    data_size_kb: usize,
}

impl UserMonkey {
    pub fn new(target: String) -> Self {
        UserMonkey {
            target,
            files: Vec::new(),
            amount_of_files: 20,
            seconds: 1,
            data_size_kb: 1000,
        }
    }

    pub async fn run(&mut self) {
        loop {
            // Get current directory list but only files which contain "banana_"
            let mut files = fs::read_dir(self.target.clone()).await.expect("Failed to read directory");
            let mut new_files: Vec<PathBuf> = Vec::new();

            while let Ok(Some(file)) = files.next_entry().await {
                let file_name = file.file_name();
                let file_name = file_name.to_str().unwrap();

                if file_name.contains("banana_") {
                    new_files.push(file.path());
                }
            }

            self.files = new_files;

            // Randomly choose an action
            let action = self.random_action();

            match action {
                0 => self.create_file().await,
                1 => self.update_file().await,
                2 => self.delete_file().await,
                _ => (),
            }

            // Delay the loop
            tokio::time::sleep(std::time::Duration::from_secs(self.seconds.into())).await;
        }
    }

    fn random_action(&self) -> u32 {
        rand::thread_rng().gen_range(0..3)
    }

    async fn create_file(&self) {
        // Early exit if there are too many files
        if self.files.len() > 100 {
            return;
        }

        // Find all numbers which are not used yet
        let mut unused_numbers: Vec<u32> = Vec::new();

        for i in 0..self.amount_of_files {
            if !self.files.iter().any(|path| path.to_str().unwrap().contains(&format!("banana_{}", i))) {
                unused_numbers.push(i);
            }
        }

        // Create a new file with a random number
        let mut rng = rand::thread_rng();
        let random_number: u32 = *unused_numbers.choose(&mut rng).unwrap();

        let file_name = format!("banana_{}.txt", random_number);

        debug!("Creating file {}", self.target.clone() + "/" + &file_name);

        let mut file = File::create(self.target.clone() + "/" + &file_name).await.expect("Failed to create file");

        // Write random data to the file
        let mut data: Vec<u8> = Vec::new();
        let random_value: u8 = rng.gen_range(0..=100);
        for _ in 0..self.data_size_kb * 1024 {
            data.push(random_value);
        }

        file.write_all(&data).await.expect("Failed to write to file");

        // Log
        info!("Created file {}", self.target.clone() + "/" + &file_name);
    }
    async fn update_file(&self) {
        if self.files.len() == 0 {
            return;
        }

        let file_path = self.random_file();

        let mut file = File::open(&file_path).await.expect("Failed to open file");

        let mut content = Vec::new();
        file.read_to_end(&mut content).await.expect("Failed to read file");

        let mut rng = rand::thread_rng();

        let mut file = File::create(&file_path).await.expect("Failed to create file");

         // Write random data to the file
         let mut data: Vec<u8> = Vec::new();
         let random_value: u8 = rng.gen_range(0..=100);
         for _ in 0..self.data_size_kb {
             data.push(random_value);
         }

         file.write_all(&data).await.expect("Failed to write to file");

        // Log
        info!("Updated file {}", file_path.to_str().unwrap());
    }

    async fn delete_file(&self) {
        if self.files.len() == 0 {
            return;
        }

        let file_path = self.random_file();

        fs::remove_file(&file_path).await.expect("Failed to delete file");

        // Log
        info!("Deleted file {}", file_path.to_str().unwrap());
    }

    fn random_file(&self) -> PathBuf {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.files.len());
        self.files[index].clone()
    }
}

use rand::prelude::SliceRandom;
use rand::Rng;
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

pub struct UserMonkey {
    path: String,
    files: Vec<PathBuf>,
    amount_of_files: u32,
}

impl UserMonkey {
    pub fn new(path: String) -> Self {
        UserMonkey {
            path,
            files: Vec::new(),
            amount_of_files: 100
        }
    }

    pub async fn run(&mut self) {
        loop {
            // Get current directory list but only files which contain "banana_"
            let mut files = fs::read_dir(self.path.clone()).await.expect("Failed to read directory");
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
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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

        let file = File::create(self.path.clone() + "/" + &file_name).await.expect("Failed to create file");
    }
    async fn update_file(&self) {
        let file_path = self.random_file();

        let mut file = File::open(&file_path).await.expect("Failed to open file");

        let mut content = Vec::new();
        file.read_to_end(&mut content).await.expect("Failed to read file");

        let mut rng = rand::thread_rng();
        let random_value: u32 = rng.gen_range(0..=100);

        let new_content = format!("Updated value: {}", random_value);

        let mut file = File::create(&file_path).await.expect("Failed to create file");
        file.write_all(new_content.as_bytes()).await.expect("Failed to write to file");
    }

    async fn delete_file(&self) {
        let file_path = self.random_file();

        fs::remove_file(&file_path).await.expect("Failed to delete file");
    }

    fn random_file(&self) -> PathBuf {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.files.len());
        self.files[index].clone()
    }
}

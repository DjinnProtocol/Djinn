use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};

use async_recursion::async_recursion;
use tokio::fs;

pub struct IndexManager {
    pub index: HashMap<String, usize>,
    pub root: String,
}

impl IndexManager {
    pub fn new(root: String) -> Self {
        IndexManager {
            index: HashMap::new(),
            root
        }
    }

    pub fn add(&mut self, key: String, value: usize) {
        self.index.insert(key, value);
    }

    pub fn get(&self, key: &String) -> Option<&usize> {
        self.index.get(key)
    }

    pub fn update(&mut self, map: HashMap<String, usize>) {
        //Merge the two maps
        for (key, value) in map {
            self.index.insert(key, value);
        }
    }

    pub async fn build(&mut self) {
        let mut index = self.build_index(self.root.clone()).await;
        // Save current timestamp
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get system time");

        let unix_timestamp = current_time.as_secs();

        // Add the index to the index manager
        index.insert("#timestamp".to_string(), unix_timestamp as usize);

        self.index = index;
    }

    #[async_recursion]
    pub async fn build_index(&mut self, directory_path: String) -> HashMap<String, usize> {
        // Build the index
        let mut index = HashMap::new();
        let mut items = fs::read_dir(directory_path).await.unwrap();

        while let Ok(Some(item)) = items.next_entry().await {
            //Check if the path is a file
            let unwrapped_item = item;
            let path_str = unwrapped_item.path().to_str().unwrap().to_string();
            let path_without_root = path_str.replace(&self.root, "/");

            if unwrapped_item.path().is_file() {
                // Check if extension is .djinn_temp
                if path_str.ends_with(".djinn_temp") {
                    continue;
                }
                //Get the file size
                let last_modified = unwrapped_item.metadata().await.unwrap().modified().unwrap();
                let last_modified_unix = last_modified.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                //Add the file name and size to the index
                index.insert(path_without_root.replace("//", "/"), last_modified_unix as usize);
            } else {
                //If the path is a directory, recursively call the function
                let sub_index = self.build_index(path_str).await;
                //Merge the two maps
                for (key, value) in sub_index {
                    index.insert(key, value);
                }
            }
        }

        index
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use tokio::fs;

    use super::*;

    #[tokio::test]
    async fn test_build_index() {
        //Setup test directory
        let test_dir = "/tmp/test_data";
        let test_file = "/tmp/test_data/test_file.txt";
        let test_sub_dir = "/tmp/test_data/test_sub_dir";
        let test_sub_file = "/tmp/test_data/test_sub_dir/test_sub_file.txt";

        //Create the test directory
        fs::create_dir_all(test_dir).await.unwrap();
        fs::create_dir_all(test_sub_dir).await.unwrap();
        fs::write(test_file, "test").await.unwrap();
        fs::write(test_sub_file, "test").await.unwrap();

        //Test
        let mut index_manager = IndexManager::new(test_dir.to_string());
        index_manager.build().await;
        //Check if the index is correct
        let mut expected_index = HashMap::new();
        let current_unix = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as usize;

        expected_index.insert(test_file.replace(test_dir, ""), current_unix);
        expected_index.insert(test_sub_file.replace(test_dir, ""), current_unix);
        expected_index.insert("#timestamp".to_string(), current_unix);

        assert_eq!(index_manager.index, expected_index);

        //Cleanup
        fs::remove_dir_all(test_dir).await.unwrap();
    }
}

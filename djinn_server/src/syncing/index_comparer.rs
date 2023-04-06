use std::{collections::HashMap};
pub enum SourceOfTruth {
    Client,
    Server,
}

pub struct IndexComparer {
    pub client_index: HashMap<String, usize>,
    pub server_index: HashMap<String, usize>,
    pub source_of_truth: SourceOfTruth,
}

impl IndexComparer {
    pub fn new(
        local_index: HashMap<String, usize>,
        remote_index: HashMap<String, usize>,
        source_of_truth: SourceOfTruth,
    ) -> Self {
        Self {
            client_index: local_index,
            server_index: remote_index,
            source_of_truth,
        }
    }

    pub fn compare(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        //METHOD -> PATH

        //Changes from client perspective
        for (key, value) in &self.client_index {
            //Check if key exists in server index
            if self.server_index.contains_key(key) {
                //Check if value is the same
                if self.server_index.get(key).unwrap() > value {
                    //Server has newer version
                    result.insert("GET".to_string(), key.to_string());
                } else {
                    //Client has newer version
                    result.insert("PUT".to_string(), key.to_string());
                }
            } else {
                if matches!(self.source_of_truth, SourceOfTruth::Client) {
                    //File does not exist on server
                    result.insert("PUT".to_string(), key.to_string());
                } else {
                   //File delete
                    result.insert("DELETE".to_string(), key.to_string());
                }
            }
        }

        //Check for files server has that client does not
        for (key, _) in &self.server_index {
            if !self.client_index.contains_key(key) {
                if matches!(self.source_of_truth, SourceOfTruth::Client) {
                    //File delete
                    result.insert("SELF_DELETE".to_string(), key.to_string());
                } else {
                    //File does not exist on client
                    result.insert("GET".to_string(), key.to_string());
                }
            }
        }

        result
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_add() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 123);

        let mut server_index = HashMap::new();

        let comparer = IndexComparer::new(client_index, server_index, SourceOfTruth::Client);
        let result = comparer.compare();

        assert_eq!(result.get("PUT").unwrap(), "test.txt");
    }

    #[test]
    fn test_client_delete() {
        let mut client_index = HashMap::new();

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 123);

        let comparer = IndexComparer::new(client_index, server_index, SourceOfTruth::Client);
        let result = comparer.compare();

        assert_eq!(result.get("SELF_DELETE").unwrap(), "test.txt");
    }

    #[test]
    fn test_server_add() {
        let mut client_index = HashMap::new();

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 123);

        let comparer = IndexComparer::new(client_index, server_index, SourceOfTruth::Server);
        let result = comparer.compare();

        assert_eq!(result.get("GET").unwrap(), "test.txt");
    }

    #[test]
    fn test_server_delete() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 123);

        let mut server_index = HashMap::new();

        let comparer = IndexComparer::new(client_index, server_index, SourceOfTruth::Server);
        let result = comparer.compare();

        assert_eq!(result.get("DELETE").unwrap(), "test.txt");
    }

    #[test]
    fn test_server_update() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 123);

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 124);

        let comparer = IndexComparer::new(client_index, server_index, SourceOfTruth::Server);
        let result = comparer.compare();

        assert_eq!(result.get("GET").unwrap(), "test.txt");
    }

    #[test]
    fn test_client_update() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 123);

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 124);

        let comparer = IndexComparer::new(client_index, server_index, SourceOfTruth::Server);
        let result = comparer.compare();

        assert_eq!(result.get("GET").unwrap(), "test.txt");
    }
}

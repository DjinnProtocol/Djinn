use std::collections::HashMap;

#[derive(Copy, Clone)]
pub enum SourceOfTruth {
    Client,
    Server,
}

pub struct IndexComparer {
    pub client_index: HashMap<String, usize>,
    pub server_index: HashMap<String, usize>,
    pub source_of_truth: SourceOfTruth,
    pub server_deleted: HashMap<String, usize>,
}

impl IndexComparer {
    pub fn new(
        client_index: HashMap<String, usize>,
        server_index: HashMap<String, usize>,
        source_of_truth: SourceOfTruth,
        server_deleted: HashMap<String, usize>,
    ) -> Self {
        Self {
            client_index,
            server_index,
            source_of_truth,
            server_deleted,
        }
    }

    pub fn compare(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        // Get timestamps
        let some_server_timestamp = self.server_index.get("#timestamp");
        let _server_timestamp = if some_server_timestamp.is_some() {
            some_server_timestamp.unwrap()
        } else {
            &0
        };

        let some_client_timestamp = self.client_index.get("#timestamp");
        let client_timestamp = if some_client_timestamp.is_some() {
            some_client_timestamp.unwrap()
        } else {
            &0
        };

        //PATH -> METHOD

        //Changes from client perspective
        for (key, timestamp) in &self.client_index {
            if &key[..1] == "#" {
                continue;
            }
            //Check if key exists in server index (file exists on server)
            if self.server_index.contains_key(key) {
                //Check if timestamp is the same
                if self.server_index.get(key).unwrap() == timestamp {
                    //File is the same
                    continue;
                } else if timestamp == &0 {
                    // Client requests delete
                    if matches!(self.source_of_truth, SourceOfTruth::Client)
                        && client_timestamp > self.server_index.get(key).unwrap()
                    {
                        //File delete
                        result.insert(key.to_string(), "SELF_DELETE".to_string());
                    } else {
                        //File does not exist on client
                        result.insert(key.to_string(), "GET".to_string());
                    }
                } else if self.server_index.get(key).unwrap() > timestamp {
                    //Server has newer version
                    result.insert(key.to_string(), "GET".to_string());
                } else {
                    //Client has newer version
                    result.insert(key.to_string(), "PUT".to_string());
                }
            } else if timestamp != &0 {
                let possible_deleted_timestamp = self.server_deleted.get(key);
                if matches!(self.source_of_truth, SourceOfTruth::Client)
                    && (possible_deleted_timestamp.is_none()
                        || possible_deleted_timestamp.unwrap() < timestamp)
                {
                    //File does not exist on server
                    result.insert(key.to_string(), "PUT".to_string());
                } else {
                    //File delete
                    result.insert(key.to_string(), "DELETE".to_string());
                }
            }
        }

        //Check for files server has that client does not
        for (key, timestamp) in &self.server_index {
            if &key[..1] == "#" {
                continue;
            }
            if !self.client_index.contains_key(key) {
                debug!("Checking for server file: {:?}", key);
                debug!("Server timestamp: {:?}", timestamp);
                debug!("Client timestamp: {:?}", client_timestamp);
                //File does not exist on client
                result.insert(key.to_string(), "GET".to_string());
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

        let server_index = HashMap::new();

        let server_deleted = HashMap::new();

        let comparer = IndexComparer::new(
            client_index,
            server_index,
            SourceOfTruth::Client,
            server_deleted,
        );
        let result = comparer.compare();

        assert_eq!(result.get("test.txt").unwrap(), "PUT");
    }

    #[test]
    fn test_client_delete() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 0);
        client_index.insert("#timestamp".to_string(), 123);

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 120);
        server_index.insert("#timestamp".to_string(), 122);

        let server_deleted = HashMap::new();

        let comparer = IndexComparer::new(
            client_index,
            server_index,
            SourceOfTruth::Client,
            server_deleted,
        );
        let result = comparer.compare();

        assert_eq!(result.get("test.txt").unwrap(), "SELF_DELETE");
    }

    #[test]
    fn test_server_add() {
        let mut client_index = HashMap::new();
        client_index.insert("#timestamp".to_string(), 123);

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 123);

        let server_deleted = HashMap::new();

        let comparer = IndexComparer::new(
            client_index,
            server_index,
            SourceOfTruth::Server,
            server_deleted,
        );
        let result = comparer.compare();

        assert_eq!(result.get("test.txt").unwrap(), "GET");
    }

    #[test]
    fn test_server_delete() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 123);

        let server_index = HashMap::new();

        let server_deleted = HashMap::new();

        let comparer = IndexComparer::new(
            client_index,
            server_index,
            SourceOfTruth::Server,
            server_deleted,
        );
        let result = comparer.compare();

        assert_eq!(result.get("test.txt").unwrap(), "DELETE");
    }

    #[test]
    fn test_server_update() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 123);

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 124);

        let server_deleted = HashMap::new();

        let comparer = IndexComparer::new(
            client_index,
            server_index,
            SourceOfTruth::Server,
            server_deleted,
        );
        let result = comparer.compare();

        assert_eq!(result.get("test.txt").unwrap(), "GET");
    }

    #[test]
    fn test_client_update() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 123);

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 124);

        let server_deleted = HashMap::new();

        let comparer = IndexComparer::new(
            client_index,
            server_index,
            SourceOfTruth::Server,
            server_deleted,
        );
        let result = comparer.compare();

        assert_eq!(result.get("test.txt").unwrap(), "GET");
    }

    #[test]
    fn test_client_same() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 123);

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 123);

        let server_deleted = HashMap::new();

        let comparer = IndexComparer::new(
            client_index,
            server_index,
            SourceOfTruth::Server,
            server_deleted,
        );
        let result = comparer.compare();

        // Check result empty
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_if_extra_info_gets_ignored() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 123);
        client_index.insert("#test.txt".to_string(), 123);

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 124);

        let server_deleted = HashMap::new();

        let comparer = IndexComparer::new(
            client_index,
            server_index,
            SourceOfTruth::Server,
            server_deleted,
        );
        let result = comparer.compare();

        assert_eq!(result.get("test.txt").unwrap(), "GET");
        assert_eq!(result.len(), 1);
    }

    #[test]
    /*
       If a client pushes an update and another client deletes it beforehand the server should not delete the file and instead
       request the client to get the file.
    */
    fn test_out_of_sync_client_delete_before_update() {
        let mut client_index = HashMap::new();
        client_index.insert("#timestamp".to_string(), 123);

        let mut server_index = HashMap::new();
        server_index.insert("test.txt".to_string(), 124);
        server_index.insert("#timestamp".to_string(), 124);

        let server_deleted = HashMap::new();

        let comparer = IndexComparer::new(
            client_index,
            server_index,
            SourceOfTruth::Client,
            server_deleted,
        );
        let result = comparer.compare();

        assert_eq!(result.get("test.txt").unwrap(), "GET");
    }

    #[test]
    /*
       If a client has a file the server doesnt anymore, the server should delete the file on the client.
    */
    fn test_out_of_sync_client_put_after_delete() {
        let mut client_index = HashMap::new();
        client_index.insert("test.txt".to_string(), 123);
        client_index.insert("#timestamp".to_string(), 123);

        let mut server_index = HashMap::new();
        server_index.insert("#timestamp".to_string(), 120);

        let mut server_deleted = HashMap::new();
        server_deleted.insert("test.txt".to_string(), 124);

        let comparer = IndexComparer::new(
            client_index,
            server_index,
            SourceOfTruth::Client,
            server_deleted,
        );
        let result = comparer.compare();

        assert_eq!(result.get("test.txt").unwrap(), "DELETE");
    }
}

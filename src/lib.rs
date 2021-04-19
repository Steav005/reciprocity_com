use serde::{Deserialize, Serialize};

mod host;
mod client;

#[derive(Debug, Deserialize, Serialize, Eq)]
pub enum Message {

}

impl PartialEq for Message{
    fn eq(&self, other: &Self) -> bool {
        match (self, other){
            //TODO Add Others
            (_, _) => false
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        todo!()
    }
}
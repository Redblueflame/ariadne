use std::collections::HashMap;
use clickhouse_rs::types::Value;
use clickhouse_rs::Block;
use clickhouse_rs::types::Column;
pub mod download;
pub mod visit;

pub trait Serializable {
    fn export(self, block: &mut AriadneBlock);
}

pub struct AriadneBlock {
    internal: HashMap<String, Vec<Value>>
}
impl AriadneBlock {
    pub fn add_element(&mut self, col_id: &str, value: Value) {
        if !self.internal.contains_key(col_id) {
            self.internal.insert(col_id.to_string(), vec![]);
        }
        self.internal.get_mut(col_id).unwrap().push(value);
    }
    pub fn get_col(&self, col_id: &str) -> Option<&Vec<Value>> {
        self.internal.get(col_id)
    }
    pub fn generate_block(&self) -> Block {
        let mut result = Block::new();
        /*for (key, val) in self.internal {
            //result = result.column(&*key, val);
        }
        */
        result
    }
}
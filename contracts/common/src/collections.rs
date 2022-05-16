use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    env,
};

const ERR_ELEMENT_DESERIALIZATION: &str = "Cannot deserialize element";

pub trait KeyValueAccess<Key, Value, Index>
where
    Key: BorshSerialize + BorshDeserialize,
    Value: BorshSerialize + BorshDeserialize,
{
    fn index(
        &self,
        index: Index,
    ) -> Option<(Key, Value)>;
}

impl<Key, Value> KeyValueAccess<Key, Value, u64> for UnorderedMap<Key, Value>
where
    Key: BorshSerialize + BorshDeserialize,
    Value: BorshSerialize + BorshDeserialize,
{
    fn index(
        &self,
        index: u64,
    ) -> Option<(Key, Value)> {
        let keys = self.keys_as_vector();
        let values = self.values_as_vector();

        let key = keys
            // use raw operators because in this way we avoid
            // borrowing problems that comes with using iter() in a
            // loop that also tries to take self by ref mut
            .get_raw(index)
            .map(|bytes| {
                Key::try_from_slice(&bytes)
                    .unwrap_or_else(|_err| env::panic_str(ERR_ELEMENT_DESERIALIZATION))
            })?;

        let value: Value = values
            // use raw operators because in this way we avoid
            // borrowing problems that comes with using iter() in a
            // loop that also tries to take self by ref mut
            .get_raw(index)
            .map(|bytes| {
                Value::try_from_slice(&bytes)
                    .unwrap_or_else(|_err| env::panic_str(ERR_ELEMENT_DESERIALIZATION))
            })?;

        Some((key, value))
    }
}

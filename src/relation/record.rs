use crate::common::constants::BlockIdT;
use crate::relation::schema::Schema;

/// A database record with variable-length attributes.
///
/// The initial section of the record contains a null bitmap which represents
/// which attributes are null and should be ignored.
///
/// The next section of a record contains fixed-length values. Data types
/// such as numerics, booleans, and dates are encoded as is, while variable-
/// length data types such as varchars are encoded as a offset/length pair.
///
/// The actual variable-length data is stored consecutively after the initial
/// fixed-length section and null bitmap.
///
/// Data format:
/// ------------------------------------------------------------
///  NULL BITMAP | FIXED-LENGTH VALUES | VARIABLE-LENGTH VALUES
/// ------------------------------------------------------------
///
/// Metadata regarding a record is stored in a system catalog in a separate
/// database block.

pub struct Record {
    pub id: RecordId,
    pub data: Vec<u8>,
}

impl Record {
    pub fn new(block_id: BlockIdT, slot_index: u32, tmp: Vec<u8>) -> Self {
        Self {
            id: RecordId {
                block_id,
                slot_index,
            },
            data: tmp,
        }
    }

    pub fn len(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn get_column_value(&self, idx: u32, schema: &Schema) {}
}

/// A database record identifier comprised of the block ID and slot index that
/// the record is located at.
pub struct RecordId {
    pub block_id: BlockIdT,
    pub slot_index: u32,
}

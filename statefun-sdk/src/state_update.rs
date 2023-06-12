use crate::ValueSpecBase;

#[derive(Debug)]
pub (crate) enum StateUpdate {
    Update(ValueSpecBase, Vec<u8>),
    Delete(ValueSpecBase),
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
	Arrow,
	Magic,
	Boomerang,
	Unknown,
	Boulder
}
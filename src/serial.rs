use super::Record;

#[derive(Clone, Debug)]
#[repr(packed)]
pub struct Serial {
    pub record: Record,
    pub kind: u32,
    pub baseaddr: u32,
    pub baud: u32,
    pub regwidth: u32,
    pub input_hertz: u32,
    pub uart_pci_addr: u32,
}

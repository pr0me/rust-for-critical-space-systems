use crate::libcsp_ffi;
use num_enum::TryFromPrimitive;

/// Cubesat Space Protocol Error Types
#[repr(i32)]
#[derive(Copy, Clone)]
pub enum CspError {
    None = 0,
    Nomem = -1,
    Inval = -2,
    Timedout = -3,
    Used = -4,
    Notsup = -5,
    Busy = -6,
    Already = -7,
    Reset = -8,
    Nobufs = -9,
    Tx = -10,
    Driver = -11,
    Again = -12,
    Nosys = -38,
    Hmac = -100,
    Crc32 = -102,
    Sfp = -103,
}

/// CAN Fragmentation Protocol (CFP) Type
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
pub enum CfpFrameType {
    /// First CFP fragment of a CSP packet
    CfpBegin = 0,
    /// Remaining CFP fragment(s) of a CSP packet
    CfpMore = 1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union LayerData {
    /// Only used on layer 3 (RDP)
    pub rdp_data: RdpData,
    /// Only used on interface RX/TX (layer 2)
    pub rx_tx_data: RxTxData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union PacketData {
    pub data: [u8; libcsp_ffi::CSP_BUFFER_SIZE as _],
    pub data16: [u16; (libcsp_ffi::CSP_BUFFER_SIZE / 2) as _],
    pub data32: [u32; (libcsp_ffi::CSP_BUFFER_SIZE / 4) as _],
}

/// Data Struct only used on layer 3 (RDP)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RdpData {
    /// EACK quarantine period
    pub rdp_quarantine: u32,
    /// Time the message was sent
    pub timestamp_tx: u32,
    /// Time the message was received
    pub timestamp_rx: u32,
    /// Associated connection (used in RDP queue); actually a *csp_conn_s
    pub conn: *mut libcsp_ffi::csp_conn_s,
}

/// Data Struct only used on interface RX/TX (layer 2)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RxTxData {
    /// Received bytes
    pub rx_count: u16,
    /// Remaining packets
    pub remain: u16,
    /// Connection CFP identification number
    pub cfpid: u32,
    /// Timestamp in ms for last use of buffer
    pub last_used: u32,
    pub frame_begin: *mut u8,
    pub frame_length: u16,
}

/// CSP packet - constructed to fit with all interface and protocols to prevent the need to copy data (zero copy)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CspPacket {
    pub layer: LayerData,
    pub length: u16,
    pub id: libcsp_ffi::csp_id_t,
    pub next: *mut CspPacket,
    pub header: [u8; libcsp_ffi::CSP_PACKET_PADDING_BYTES as _],
    pub data: PacketData,
}

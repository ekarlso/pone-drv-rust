pub enum CTapCHIDCmd {
    Msg = 0x03,
    Cbot = 0x10,
    Init = 0x06,
    Ping = 0x01,
    Cancel = 0x11,
    Error = 0x3f,
    KeepAlive = 0x3b,
    Wink = 0x08,
    Lock = 0x04,
}

pub enum CTapHIDCapabilities {
    Wink = 0x01, // not defined for BLE
    Error = 0x04,
    Nmsg = 0x08, // PONE OffPAD currently only supports FIDO2, not U2F, so this will be set for now
}

pub enum CTapBLECommand {
    Ping = 0x01,
    KeepAlive = 0x82,
    Msg = 0x83,
    Cancel = 0xbe,
    Error = 0xbf,
}

// ee: https://fidoalliance.org/specs/fido-v2.1-rd-20210309/fido-client-to-authenticator-protocol-v2.1-rd-20210309.html#ble-constantstra
pub enum CTapBLEError {
    InvalidCommand = 0x01,
    InvalidPAR = 0x02,
    InvalidLength = 0x03,
    InvalidSequence = 0x04,
    RequestTimeout = 0x05,
    Busy = 0x06,
    LockRequired = 0x0a,   // Only relevant if HID
    InvalidChannel = 0x0b, // Only relevant if HID
    Other = 0x7f,
}

// Status codes - https://fidoalliance.org/specs/fido-v2.1-rd-20210309/fido-client-to-authenticator-protocol-v2.1-rd-20210309.html#error-responses
pub enum CTapStatus {
    // The command is not a valid CTAP command.
    ErrInvalidCommand = 0x01,
    // Invalid message sequencing.
    ErrInvalidSeq = 0x04,
    // Command not allowed on this cid.
    ErrInvalidChannel = 0x0b,
    // Other unspecified error.
    ErrOther = 0x7f,
}

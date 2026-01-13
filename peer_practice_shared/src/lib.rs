pub use peer_practice_messages::Envelope;
use peer_practice_messages::Version;
use peer_practice_messages::v2026_01_11::messages::ClientToServer;
pub use peer_practice_messages::v2026_01_11::*;

pub fn create_envelope(msg: ClientToServer) -> Envelope<ClientToServer> {
    Envelope {
        version: Version::V2026_01_11,
        data: msg,
    }
}

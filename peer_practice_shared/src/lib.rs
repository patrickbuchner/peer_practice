pub use peer_practice_messages::Envelope;
use peer_practice_messages::Version;
use peer_practice_messages::v2026_02_07::messages::ClientToServer;
pub use peer_practice_messages::v2026_02_07::*;

pub fn create_envelope(msg: ClientToServer) -> Envelope<ClientToServer> {
    Envelope {
        version: Version::V2026_02_07,
        data: msg,
    }
}

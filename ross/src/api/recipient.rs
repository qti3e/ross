use super::EditorMessage;

/// A type that can be used as the message receiver associated with a session.
pub trait Recipient {
    type SerializedType;

    /// When broadcasting a message we want to avoid serializing the message
    /// once for each session/client, so we have abstracted out this method
    /// here so we can call it only once before actually attempting to send
    /// the message to all of the active clients.
    fn serialize(message: &EditorMessage) -> Self::SerializedType;

    /// Called by editor to send the message to the client.
    fn send(&mut self, data: &Self::SerializedType);
}

/// An special recipient that ignores the messages.
impl Recipient for () {
    type SerializedType = ();

    #[inline]
    fn serialize(_: &EditorMessage) -> Self::SerializedType {
        ()
    }

    #[inline]
    fn send(&mut self, _: &Self::SerializedType) {}
}

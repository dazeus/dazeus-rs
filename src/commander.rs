use std::sync::Future;
use super::event::{Event, EventType};
use super::listener::ListenerHandle;
use super::response::Response;
use super::scope::Scope;
use super::request::ConfigGroup;

/// Methods that need to be implemented for sending commands to the server
pub trait Commander {
    /// Subscribe to an event type and call the callback function every time such an event occurs.
    fn subscribe<F>(&self, event: EventType, callback: F) -> (ListenerHandle, Future<Response>)
        where F: FnMut(Event);

    /// Subscribe to a command and call the callback function every time such a command occurs.
    fn subscribe_command<F>(&self, command: &str, callback: F) -> (ListenerHandle, Future<Response>)
        where F: FnMut(Event);

    /// Unsubscribe a listener for some event.
    fn unsubscribe(&self, handle: ListenerHandle) -> Future<Response>;

    /// Remove all subscriptions for a specific event type.
    fn unsubscribe_all(&self, event: EventType) -> Future<Response>;

    /// Check if there is any active listener for the given event type
    fn has_any_subscription(&self, event: EventType) -> bool;

    /// Retrieve the networks the bot is connected to.
    fn networks(&self) -> Future<Response>;

    /// Retrieve the channels the bot is in for a given network.
    fn channels(&self, network: &str) -> Future<Response>;

    /// Send a message to a specific channel using the PRIVMSG method.
    fn message(&self, network: &str, channel: &str, message: &str) -> Future<Response>;

    /// Send a CTCP NOTICE to a specific channel.
    fn notice(&self, network: &str, channel: &str, message: &str) -> Future<Response>;

    /// Send a CTCP REQUEST to a specific channel.
    fn ctcp(&self, network: &str, channel: &str, message: &str) -> Future<Response>;

    /// Send a CTCP REPLY to a specific channel.
    fn ctcp_reply(&self, network: &str, channel: &str, message: &str) -> Future<Response>;

    /// Send a CTCP ACTION to a specific channel
    fn action(&self, network: &str, channel: &str, message: &str) -> Future<Response>;

    /// Send a request for the list of nicks in a channel.
    ///
    /// Note that the response will not contain the names data, instead listen for a names event.
    /// The Response will only indicate whether or not the request has been submitted successfully.
    /// The server may respond with an `EventType::Names` event any time after this request has
    /// been submitted.
    fn send_names(&self, network: &str, channel: &str) -> Future<Response>;

    /// Send a request for a whois of a specific nick on some network.
    ///
    /// Note that the response will not contain the whois data, instead listen for a whois event.
    /// The Response will only indicate whether or not the request has been submitted successfully.
    /// The server may respond with an `EventType::Whois` event any time after this request has
    /// been submitted.
    fn send_whois(&self, network: &str, nick: &str) -> Future<Response>;

    /// Try to join a channel on some network.
    fn join(&self, network: &str, channel: &str) -> Future<Response>;

    /// Try to leave a channel on some network.
    fn part(&self, network: &str, channel: &str) -> Future<Response>;

    /// Retrieve the nickname of the bot on the given network.
    fn nick(&self, network: &str) -> Future<Response>;

    /// Send a handshake to the DaZeus core.
    fn handshake(&self, name: &str, version: &str, config: Option<&str>) -> Future<Response>;

    /// Retrieve a config value from the DaZeus config.
    fn get_config(&self, name: &str, group: ConfigGroup) -> Future<Response>;

    /// Retrieve the character that is used by the bot for highlighting.
    fn get_highlight_char(&self) -> Future<Response>;

    /// Retrieve a property stored in the bot database.
    fn get_property(&self, name: &str, scope: Scope) -> Future<Response>;

    /// Set a property to be stored in the bot database.
    fn set_property(&self, name: &str, value: &str, scope: Scope) -> Future<Response>;

    /// Remove a property stored in the bot database.
    fn unset_property(&self, name: &str, scope: Scope) -> Future<Response>;

    /// Retrieve a list of keys starting with the common prefix with the given scope.
    fn get_property_keys(&self, prefix: &str, scope: Scope) -> Future<Response>;

    /// Set a permission to either allow or deny for a specific scope.
    fn set_permission(&self, permission: &str, allow: bool, scope: Scope) -> Future<Response>;

    /// Retrieve whether for some scope the given permission was set.
    ///
    /// Will return the default if it was not.
    fn has_permission(&self, permission: &str, default: bool, scope: Scope) -> Future<Response>;

    /// Remove a set permission from the bot.
    fn unset_permission(&self, permission: &str, scope: Scope) -> Future<Response>;

    /// Send a whois request and wait for an event that answers this request (blocking).
    ///
    /// Note that the IRC server may not respond to the whois request (if it has been configured
    /// this way), in which case this request will block forever.
    fn whois(&self, network: &str, nick: &str) -> Event;

    /// Send a names request and wait for an event that answers this request (blocking).
    ///
    /// Note that the IRC server may not respond to the names request (if it has been configured
    /// this way), in which case this request will block forever.
    fn names(&self, network: &str, channel: &str) -> Event;

    /// Send a reply in response to some event.
    ///
    /// Note that not all types of events can be responded to. Mostly message type events
    /// concerning some IRC user can be responded to. Join events can also be responded to.
    fn reply(&self, event: &Event, message: &str, highlight: bool) -> Future<Response>;

    /// Send a reply (as a ctcp action) in response to some event.
    ///
    /// Note that not all types of events can be responded to. Mostly message type events
    /// concerning some IRC user can be responded to. Join events can also be responded to.
    fn reply_with_action(&self, event: &Event, message: &str) -> Future<Response>;

    /// Send a reply (as a notice) in response to some event.
    ///
    /// Note that not all types of events can be responded to. Mostly message type events
    /// concerning some IRC user can be responded to. Join events can also be responded to.
    fn reply_with_notice(&self, event: &Event, message: &str) -> Future<Response>;
}

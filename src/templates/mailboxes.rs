use askama::Template;
use crate::models::Mailbox;

#[derive(Template)]
#[template(path = "mailboxes/list.html")]
pub struct MailboxListTemplate {
    pub mailboxes: Vec<Mailbox>,
}

#[derive(Template)]
#[template(path = "mailboxes/show.html")]
pub struct MailboxShowTemplate {
    pub mailbox: Mailbox,
}

#[derive(Template)]
#[template(path = "mailboxes/form.html")]
pub struct MailboxFormTemplate {
    pub mailbox: Option<Mailbox>,
} 

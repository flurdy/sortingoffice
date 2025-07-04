use askama::Template;
use crate::models::{Mailbox, MailboxForm};

#[derive(Template)]
#[template(path = "mailboxes/list.html", escape = "html")]
pub struct MailboxListTemplate<'a> {
    pub title: &'a str,
    pub mailboxes: Vec<Mailbox>,
}

#[derive(Template)]
#[template(path = "mailboxes/show.html", escape = "html")]
pub struct MailboxShowTemplate<'a> {
    pub title: &'a str,
    pub mailbox: Mailbox,
}

#[derive(Template)]
#[template(path = "mailboxes/form.html", escape = "html")]
pub struct MailboxFormTemplate<'a> {
    pub title: &'a str,
    pub mailbox: Option<Mailbox>,
    pub form: MailboxForm,
    pub error: Option<String>,
} 

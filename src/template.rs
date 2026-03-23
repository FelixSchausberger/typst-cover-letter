pub struct TemplateArgs<'a> {
    /// Company name (for recipient block)
    pub company: &'a str,
    /// Street and city portion of address, e.g. "Pestalozzistraße 32\ 80469 München"
    pub address: &'a str,
    /// Optional contact person, prepended to recipient block
    pub contact: Option<&'a str>,
    /// Job position / title
    pub position: &'a str,
    /// "de" or "en"
    pub lang: &'a str,
    /// Date as literal string, e.g. "23.03.2026"
    pub date: &'a str,
}

pub fn render(args: &TemplateArgs) -> String {
    // Build recipient block: [Contact\ Company\ Street\ City]
    let company_address = if args.address.is_empty() {
        args.company.to_string()
    } else {
        format!("{}\\ {}", args.company, args.address)
    };
    let recipient = match args.contact {
        Some(contact) => format!("{}\\ {}", contact, company_address),
        None => company_address,
    };

    let subject = match args.lang {
        "de" => format!("Bewerbung als {}", args.position),
        _ => format!("Application for {}", args.position),
    };

    let salutation_hint = match (args.lang, args.contact) {
        ("de", Some(contact)) => {
            let last = contact.split_whitespace().last().unwrap_or(contact);
            format!("Sehr geehrte/r Herr/Frau {},", last)
        }
        ("de", None) => "Sehr geehrte Damen und Herren,".to_string(),
        (_, Some(contact)) => format!("Dear {},", contact),
        (_, None) => "Dear Hiring Team,".to_string(),
    };

    format!(
        r#"#import "@local/cover-letter:0.1.0": cover-letter

#show: cover-letter.with(
  recipient: [{recipient}],
  date: [{date}],
  subject: [{subject}],
)

// {salutation}
"#,
        recipient = recipient,
        date = args.date,
        subject = subject,
        salutation = salutation_hint,
    )
}

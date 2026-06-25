pub struct TemplateArgs<'a> {
    /// Company name (for recipient block)
    pub company: &'a str,
    /// Street and city portion of address, e.g. "Pestalozzistraße 32\ 80469 München"
    pub address: &'a str,
    /// Optional contact person, e.g. "Dr. Max Mustermann"
    pub contact: Option<&'a str>,
    /// Gender of contact: "male" or "female" (None when no contact)
    pub contact_gender: Option<&'a str>,
    /// Job position / title
    pub position: &'a str,
    /// "de" or "en"
    pub lang: &'a str,
    /// Date as literal string, e.g. "23.03.2026"
    pub date: &'a str,
    /// Sender name, e.g. "Felix Schausberger"
    pub sender_name: &'a str,
    /// Sender street, e.g. "Arsenalstraße 12/1403"
    pub sender_street: &'a str,
    /// Sender city, e.g. "1100 Wien"
    pub sender_city: &'a str,
}

pub fn render(args: &TemplateArgs) -> String {
    // Build recipient block: [z.H. Contact\ Company\ Street\ City]
    let company_address = if args.address.is_empty() {
        args.company.to_string()
    } else {
        format!("{}\\ {}", args.company, args.address)
    };
    let recipient = match (args.lang, args.contact, args.contact_gender) {
        ("de", Some(contact), Some("female")) => {
            format!("z.H. Frau {}\\ {}", contact, company_address)
        }
        ("de", Some(contact), Some("male")) => {
            format!("z.H. Herr {}\\ {}", contact, company_address)
        }
        (_, Some(contact), _) => format!("{}\\ {}", contact, company_address),
        (_, None, _) => company_address,
    };

    let subject = match args.lang {
        "de" => format!("Bewerbung als {}", args.position),
        _ => format!("Application for {}", args.position),
    };

    let salutation = match (args.lang, args.contact, args.contact_gender) {
        ("de", Some(contact), Some("female")) => {
            let last = contact.split_whitespace().last().unwrap_or(contact);
            format!("Sehr geehrte Frau {},", last)
        }
        ("de", Some(contact), Some("male")) => {
            let last = contact.split_whitespace().last().unwrap_or(contact);
            format!("Sehr geehrter Herr {},", last)
        }
        ("de", _, _) => {
            let team = args
                .company
                .split_whitespace()
                .next()
                .unwrap_or(args.company);
            format!("Sehr geehrtes {}-Team,", team)
        }
        (_, Some(contact), Some("female")) => {
            let last = contact.split_whitespace().last().unwrap_or(contact);
            format!("Dear Ms. {},", last)
        }
        (_, Some(contact), Some("male")) => {
            let last = contact.split_whitespace().last().unwrap_or(contact);
            format!("Dear Mr. {},", last)
        }
        (_, _, _) => {
            let team = args
                .company
                .split_whitespace()
                .next()
                .unwrap_or(args.company);
            format!("Dear {} Team,", team)
        }
    };

    let sender_block = format!(
        r"[{name} \ {street} \ {city}]",
        name = args.sender_name,
        street = args.sender_street,
        city = args.sender_city,
    );

    format!(
        r#"#import "@local/cover-letter:0.1.0": cover-letter

#show: cover-letter.with(
  sender: {sender},
  name: [{name}],
  recipient: [{recipient}],
  date: [{date}],
  subject: [{subject}],
)

{salutation}
"#,
        sender = sender_block,
        name = args.sender_name,
        recipient = recipient,
        date = args.date,
        subject = subject,
        salutation = salutation,
    )
}

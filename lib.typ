#import "@preview/appreciated-letter:0.1.0": letter as _letter

#let _defaults = toml("defaults.toml")

#let cover-letter(
  recipient: none,
  date: none,
  subject: none,
  sender: auto,
  name: auto,
  body,
) = {
  let resolved-sender = if sender == auto {
    let s = _defaults.sender
    [#s.name \ #s.street \ #s.city]
  } else {
    sender
  }

  let resolved-name = if name == auto {
    [#_defaults.sender.name]
  } else {
    name
  }

  show: _letter.with(
    sender: resolved-sender,
    recipient: recipient,
    date: date,
    subject: subject,
    name: resolved-name,
  )

  body
}

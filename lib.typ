#import "@preview/appreciated-letter:0.1.0": letter as _letter

#let cover-letter(
  recipient: none,
  date: none,
  subject: none,
  sender: none,
  name: none,
  body,
) = {
  let resolved-sender = if sender == none { [] } else { sender }
  let resolved-name = if name == none { [] } else { name }

  show: _letter.with(
    sender: resolved-sender,
    recipient: recipient,
    date: date,
    subject: subject,
    name: resolved-name,
  )

  body
}

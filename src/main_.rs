// remove later
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate termion;

mod terminal;
use terminal::Terminal;

use std::
{
  io::
  {
    self,
    stdin,
    stdout,
    Write,
  },
  sync::
  {
    mpsc::
    {
      channel,
      Receiver,
      Sender,
    },
  },
  thread,
};
use termion::
{
  async_stdin,
  clear,
  color,
  cursor,
  event::
  {
    Event,
    Key,
    MouseButton,
    MouseEvent,
  },
  input::
  {
    MouseTerminal,
    TermRead,
  },
  raw::
  {
    IntoRawMode,
  },
  style,
};

fn main()
{
  let terminal = Terminal::new();
/*
  let terminal = Terminal::new();
  terminal.run();
  loop
  {
    let event = terminal.rxEvents.try_recv();
    match event.unwrap()
    {
      Event::Key(Key::Esc) => break,
    }
  }
*/
}

/*
          match event.unwrap()
          {
            Event::Key(Key::Esc) =>
            {
              
            },
            Event::Mouse(MouseEvent::Press(button, posX, posY)) =>
            {
            },
            Event::Mouse(MouseEvent::Release(posX, posY)) =>
            {
            },
            Event::Mouse(MouseEvent::Hold(posX, posY)) =>
            {
            },
            Event::Unsupported(u) =>
            {
            }
            _ => {}
          }
*/
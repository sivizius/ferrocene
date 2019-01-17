extern crate termion;

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


/*
impl Terminal
{
  pub fn new
  (
  )
  {
    let ( sizeX, sizeY ): ( u32, u32 )
          = termion::terminal_size().unwrap();
    let ( txEvents, rxEvents ): ( Sender<Event>, Receiver<Event> )
          = channel();
    Self
    {
      stdin:    io::stdin(),
      stdout:   MouseTerminal::from(io::stdout().into_raw_mode().unwrap()),
      sizeX:    stdoutWidth,
      sizeY:    stdoutHeight,
      cursorX:  1,
      cursorY:  1,
      txEvents: txEvents,
      rxEvents: rxEvents,
    }
  }
  pub fn run
  (
    &mut self
  )
  {
    self.stdinHandler
      = thread::spawn
    (
      move ||
      {
        for event in self.stdin.events()
        {
          txEvents.send(event.unwrap());
        }
      }
    );
    write!
    (
      self.stdout,
      "\x1b[22;0t\x1b]0;{}\x07{}{}{}",
      "Hello World",
      termion::clear::All,
      termion::cursor::Goto(1, 1),
      termion::cursor::Hide,
    ).unwrap();
    stdout.flush().unwrap();
  }
  pub fn setWindowTitle
  (
    &self,
    title: &str
  )
  {
    write!
    (
      self.stdout,
      "\x1b[22;0t\x1b]0;{}\x07",
      title
    ).unwrap();
  }
}

impl Drop for Terminal
{
  fn drop
  (
    &mut self
  )
  {
    write!
    (
      self.stdout,
      "\x1bc",
    ).unwrap();
  }
}
*/
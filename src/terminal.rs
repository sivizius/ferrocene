pub type BufferUID = usize;
pub enum BufferOrientation
{
  North,
  South,
  East,
  West,
}

pub enum TilingMode
{
  AbsoluteResizeable,
  BuffersAreWindows,
  FocusedBufferFull,
  FocusedBufferOriented(BufferOrientation),
}

pub struct TerminalScreen
{
  tilingMode:                           TilingMode,
  listOfBuffers:                        Vec<BufferUID>,
  rootOfBuffers:                        BufferUID,
}

pub const TextBufferNull: u8            =                                       0b00000000;
pub const TextBufferWrapAround: u8      =                                       0b00000001;

pub struct TextBuffer
{
  pub flags:                            u8,
  pub chars:                            String,
}

/*
impl TextBuffer
{
  pub fn new
  (
    chars:                              &'static str,
    flags:                              u8,
  ) -> RwLock<BufferType>
  {
    RwLock::new
    (
      BufferType::TextBuffer
      (
        Self
        {
          chars:                          String::from(chars),
          flags:                          1,
        }
      )
    )
  }
}
*/

pub enum BufferType
{
  TextBuffer(TextBuffer),
  SixelBuffer,
  BitMapBuffer,
  ScreenBuffer(TerminalScreen),
}

impl BufferType
{
  pub fn newTextBuffer
  (
    chars:  &str,
    flags:  u8
  ) -> BufferType
  {
    BufferType::TextBuffer
    (
      TextBuffer
      {
        chars: String::from(chars),
        flags: flags
      }
    )
  }
}

pub struct Display
{
  root:                                 BufferUID,
}

pub struct TerminalBuffer
{
  posX:                                 u16,
  posY:                                 u16,
  sizeX:                                u16,
  sizeY:                                u16,
  minX:                                 u16,
  minY:                                 u16,
  maxX:                                 u16,
  maxY:                                 u16,
  cursorX:                              u16,
  cursorY:                              u16,
  buffer:                               BufferType,
}

pub struct Terminal
{
  stdout:                               termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>,
  pub sizeX:                            u16,
  pub sizeY:                            u16,
  cursorX:                              u16,
  cursorY:                              u16,
  stdinHandler:                         Option<std::thread::JoinHandle<()>>,
  pub events:                           Option<std::sync::mpsc::Receiver<termion::event::Event>>,
  lstOfBuffers:                         Vec<Option<TerminalBuffer>>,
  mapOfBuffers:                         Box<[u32]>,
  //displays:                             Vec<Display>,
  root:                                 BufferUID,
}

impl Terminal
{
  pub fn new
  (
  ) -> Self
  {
    let mut stdout
    = MouseTerminal::from(io::stdout().into_raw_mode().unwrap());
    let ( sizeX, sizeY ): ( u16, u16 )
    = termion::terminal_size().unwrap();
    let ( txEvents, rxEvents ): ( Sender<Event>, Receiver<Event> )
    = channel();
    write!
    (
      stdout,
      "\x1b[22;0t\x1b]0;{}\x07{}{}{}",
      "Hello World",
      termion::clear::All,
      termion::cursor::Goto(1, 1),
      termion::cursor::Hide,
    ).unwrap();
    Self
    {
      stdout:                           stdout,
      sizeX:                            sizeX,
      sizeY:                            sizeY,
      cursorX:                          1,
      cursorY:                          1,
      stdinHandler:                     Some
      (
        thread::spawn
        (
          move ||
          {
            let stdin = io::stdin();
            for event in stdin.events()
            {
              if event.is_ok()
              {
                txEvents.send(event.unwrap()).unwrap();
              }
            }
          }
        )
      ),
      events:                           Some ( rxEvents ),
      lstOfBuffers:                     Vec::with_capacity( 0xffffffff ),
      mapOfBuffers:                     Vec::with_capacity( ( sizeX * sizeY ) as usize ).into_boxed_slice(),
      root:                             0,
    }
  }

  pub fn newDisplay
  (
    &mut self,
    root:                               BufferUID,
  )
  {
    self.root = root;
  }
  
  pub fn newBuffer
  (
    &mut self,
    posX:                               u16,
    posY:                               u16,
    sizeX:                              u16,
    sizeY:                              u16,
    minX:                               u16,
    minY:                               u16,
    maxX:                               u16,
    maxY:                               u16,
    cursorX:                            u16,
    cursorY:                            u16,
    buffer:                             BufferType,
  ) -> BufferUID
  {
    self.lstOfBuffers.push
    (
      Some
      (
        TerminalBuffer
        {
          posX:                         posX + 1,
          posY:                         posY + 1,
          sizeX:                        sizeX,
          sizeY:                        sizeY,
          minX:                         minX,
          minY:                         minY,
          maxX:                         maxX,
          maxY:                         maxY,
          cursorX:                      cursorX,
          cursorY:                      cursorY,
          //buffer:                       buffer,
        }
      )
    );
    self.lstOfBuffers.len() - 1
  }
  
  pub fn addBufferToScreen
  (
    &mut self,
    screen:                             BufferUID,
    buffer:                             BufferUID,
  )
  {
    /*if let Some(ref mut r) = self.lstOfBuffers[screen]
    {
      if let BufferType::ScreenBuffer(ref mut screen) = r.buffer
      {
        screen.listOfBuffers.push(buffer);
      }
      else
      {
        //handle error
      }
    }
    else
    {
      //handle error
    }*/
  }
  
  pub fn render
  (
    &mut self
  )
  {
    /*
    if let Some(ref r) = self.lstOfBuffers[self.root]
    {
      match &r.buffer
      {
        BufferType::ScreenBuffer(screen) =>
        {
          match &screen.tilingMode
          {
            TilingMode::AbsoluteResizeable =>
            {
              for buffer in screen.listOfBuffers.iter()
              {
                if let Some(ref b) = self.lstOfBuffers[*buffer]
                {
                  match &b.buffer
                  {
                    BufferType::TextBuffer(textBuffer) =>
                    {
                      write!
                      (
                        self.stdout,
                        "{}",
                        termion::cursor::Goto
                        (
                          b.posX, b.posY
                        ),
                      ).unwrap();
                      let chars = textBuffer.chars.upgrade();
                      if let Some(ref c) = chars
                      {
                        for char in c.graphemes(true)
                        {
                          if char == "\n"
                          {
                            write!
                            (
                              self.stdout,
                              "{}",
                              termion::cursor::Goto
                              (
                                b.posX, b.posY
                              ),
                            ).unwrap();
                          }
                          else
                          {
                            write!
                            (
                              self.stdout,
                              "{}",
                              char
                            ).unwrap();
                          }
                        }
                        drop(chars);
                      }
                    },
                    _ =>
                    {
                      
                    }
                  }
/*                  for char in graphemes()
                  */
                }
                else
                {
                  //handle error
                }
              }
            },
            _ => {}
          }
          self.stdout.flush().unwrap();
        },
        _ => {}
      }
    }
    else
    {
      //handle error
    }
    */
  }
  
  /*
  pub fn newScreen
  (
    &mut self
  ) -> TerminalScreen
  {
    TerminalScreen
    {
      mapOfBuffers:   Vec::with_capacity( ( self.sizeX * self.sizeY ) as usize ).into_boxed_slice(),
    }
  }
  

  pub fn clearScreen
  (
    &mut self
  )
  {
    write!
    (
      self.stdout,
      "{}",
      termion::clear::All,
    ).unwrap();
  }
  pub fn setCursorPosition
  (
    &mut self,
    cursorX:  u16,
    cursorY:  u16,
  )
  {
    write!
    (
      self.stdout,
      "{}",
      termion::cursor::Goto
      (
        cursorX + 1,
        cursorY + 1
      ),
    ).unwrap();
  }
  
  pub fn setTerminalTitle
  (
    &mut self,
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
  */
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

      /*match *r.buffer
      {
        BufferType::ScreenBuffer(mut screen) =>
        {
          screen.listOfBuffers.push(buffer);
        },
        _ =>
        {
          //handle error
        }
      }*/



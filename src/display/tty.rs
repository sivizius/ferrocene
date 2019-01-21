use crate::
{
  display::
  {
    Display,
    DisplayFlag,
    DisplayID,
    DisplayType,
    ReadableFd,
    WriteableFd,
  },
  event::
  {
    Event,
    EventSender,
    EventType,
    MouseButton,
  },
  frame::
  {
    EditorFrame,
    FrameID,
    PixelFrame,
    PlotFrame,
    StatusFrame,
    TextFrame,
    style::
    {
      Colour,
      Style,
    },
  },
};

use std::
{
  io::
  {
    Read,
    Write,
  },
  mem,
  sync::
  {
    Arc,
    mpsc::
    {
      channel,
      Receiver,
      Sender,
    },
    Mutex,
  },
  thread::
  {
    self,
    JoinHandle,
  },
  time::
  {
    Duration,
    SystemTime
  },
};

const TTY_ESC:                     &str =                                       "\x1b";
const TTY_CSI:                     &str =                                       "\x1b[";

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Debug)]
pub enum TTYPrefix
{
  None,
  Escape,
  CSI,
  Mouse,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Debug)]
enum TTYState
{
  ExpectByte,
  Escape,
  CSI,
  ParseArgument,
}

pub struct TTYDisplay
{
  input:                                Arc<Mutex<Box<ReadableFd>>>,
  output:                               Box<WriteableFd>,
  listener:                             Option<JoinHandle<()>>,
  messages:                             Option<Sender<bool>>,
  termios:                              libc::termios,
  fcntl:                                i32,
}

impl TTYDisplay
{
  pub fn new
  (
    flags:                              DisplayFlag,
    offsX:                              isize,
    offsY:                              isize,
    cursorX:                            usize,
    cursorY:                            usize,
    mut input:                          Box<ReadableFd>,
    mut output:                         Box<WriteableFd>,
    refreshRate:                        u64,
  ) -> Result<Display, &'static str>
  {
    unsafe
    {
      let mut termios: libc::termios    =                                       mem::zeroed();
      if libc::tcgetattr( output.as_raw_fd(), &mut termios) < 0
      {
        Err("cannot get termios structure")
      }
      else
      {
        let ( sizeX, sizeY, _, _ )
                                        =                                       Self::getTerminalSize ( &mut input, &mut output, &mut termios )?;
        Ok
        (
          Display
          {
            flags:                            flags | DisplayFlag::NeedRefresh | DisplayFlag::NeedRemap,
            this:                             0,
            offsX:                            offsX,
            offsY:                            offsY,
            sizeX:                            sizeX as usize,
            sizeY:                            sizeY as usize,
            cursorX:                          cursorX,
            cursorY:                          cursorY,
            mapOfFrames:                      Arc::new(Mutex::new(None)),
            mainFrame:                        0,
            focusedFrame:                     Arc::new(Mutex::new(Box::new(0))),
            lastRefresh:                      SystemTime::now(),
            nextRefresh:                      Duration::from_nanos(refreshRate),
            display:
            DisplayType::TTY
            (
              TTYDisplay
              {
                input:                        Arc::new(Mutex::new(input)),
                output:                       output,
                listener:                     None,
                messages:                     None,
                termios:                      termios,
                fcntl:                        0,
              }
            )
          }
        )
      }
    }
  }

  pub fn getTerminalSize
  (
    input:                              &mut Box<ReadableFd>,
    output:                             &mut Box<WriteableFd>,
    termios:                            &mut libc::termios,
  ) -> Result<( u16, u16, u16, u16 ), &'static str>
  {
    unsafe
    {
      let mut winsize: libc::winsize    =                                       mem::zeroed();
      if libc::ioctl( output.as_raw_fd(), libc::TIOCGWINSZ, &mut winsize as *mut _) < 0
      //|| true
      {
        //this case is very rare:
        //  the size of the terminal has to be determined by putting the cursor to the lower right and
        //  then asking for the actual cursor position.
        //  this needs raw mode enabled, so the result can be parsed without waiting for enter.
        write!
        (
          output,
          "{}999;999H{}6n\n",
          TTY_CSI,
          TTY_CSI,
        ).unwrap();
        let mut state: TTYState         =                                       TTYState::ExpectByte;
        let mut mouseState              =                                       MouseButton::None;
        let mut listOfParameters        =                                       vec!();
        let mut currentParameter        =                                       0;
        let mut returnValue: Option<Event>
                                        =                                       None;
        let mut parameterPrefix: TTYPrefix
                                        =                                       TTYPrefix::None;
        let mut temp                    =                                       termios.clone();
        libc::cfmakeraw(&mut temp);
        if libc::tcsetattr( output.as_raw_fd(), libc::TCSAFLUSH, &mut temp) < 0
        {
          Err("cannot enter raw mode")
        }
        else
        {
          while returnValue.is_none()
          {
            if let Some(byte) = input.bytes().next()
            {
              if let Ok(byte) = byte
              {
                returnValue
                = state.nextState
                (
                  byte,
                  None,
                  0,
                  None,
                  None,
                  0,                    0,
                  &mut mouseState,
                  &mut listOfParameters,
                  &mut currentParameter,
                  &mut parameterPrefix,
                );
              }
            }
          }
          let mut temp                  =                                       termios.clone();
          if libc::tcsetattr( output.as_raw_fd(), libc::TCSAFLUSH, &mut temp) < 0
          {
            Err("cannot enter cooked mode")
          }
          else
          {
            let event: Event = returnValue.unwrap();
            if let EventType::CursorPosition = event.event
            {
              Ok
              (
                (
                  event.cursorX as u16,
                  event.cursorY as u16,
                  0,
                  0,
                )
              )
            }
            else
            {
              Err("cannot determine terminal dimensions")
            }
          }
        }
      }
      else
      {
        Ok
        (
          (
            winsize.ws_col    as u16,
            winsize.ws_row    as u16,
            winsize.ws_xpixel as u16,
            winsize.ws_ypixel as u16
          )
        )
      }
    }
  }

  pub fn changeTitle
  (
    &mut self,
    events:                             EventSender,
    display:                            DisplayID,
    title:                              String,
  )
  {
    let error
    = write!
      (
        self.output,
        "{}]0;{}\x07",
        TTY_ESC,                        title,
      );
    if !error.is_ok()
    {
      events.send
      (
        Event::new
        (
          EventType::Error("cannot send to tty"),
          display,                      0,
          0,                            0,
          MouseButton::None,
        )
      ).unwrap();
    }
    let error                           =                                       self.output.flush();
    if !error.is_ok()
    {
      events.send
      (
        Event::new
        (
          EventType::Error("cannot flush to tty"),
          display,                      0,
          0,                            0,
          MouseButton::None,
        )
      ).unwrap();
    }
  }

  pub fn turnOn
  (
    &mut self,
    events:                             EventSender,
    display:                            DisplayID,
    title:                              String,
    sizeX:                              usize,
    sizeY:                              usize,
    mapOfFrames:                        Arc<Mutex<Option<Box<[FrameID]>>>>,
    focus:                              Arc<Mutex<Box<FrameID>>>,
  )
  {
    unsafe
    {
      let mut termios                   =                                       self.termios.clone();
      libc::cfmakeraw(&mut termios);
      if libc::tcsetattr( self.output.as_raw_fd(), libc::TCSAFLUSH, &mut termios) < 0
      {
        events.send
        (
          Event::new
          (
            EventType::Error("cannot enter raw mode"),
            display,                    0,
            0,                          0,
            MouseButton::None,
          )
        ).unwrap();
      }
      if let Ok(mut input) = self.input.lock()
      {
        let fd                          =                                       input.by_ref().as_raw_fd();
        self.fcntl                      =                                       libc::fcntl(fd, libc::F_GETFL);
        if libc::fcntl( fd, libc::F_SETFL, self.fcntl | libc::O_NONBLOCK) < 0
        {
          events.send
          (
            Event::new
            (
              EventType::Error("cannot make input non-blocking"),
              display,                  0,
              0,                        0,
              MouseButton::None,
            )
          ).unwrap();
        }
      }
    }
    let error
    = write!
      (
        self.output,
        "{}]0;{}\x07{}{}J{}{};{}H{}?{}l{}?{}h{}?{}h",
        TTY_ESC,                        title,
        TTY_CSI,                        2,
        TTY_CSI,                        1,        1,
        TTY_CSI,                        25,
        TTY_CSI,                        1003,
        TTY_CSI,                        1006,
      );
    if !error.is_ok()
    {
      events.send
      (
        Event::new
        (
          EventType::Error("cannot send to tty"),
          display,                      0,
          0,                            0,
          MouseButton::None,
        )
      ).unwrap();
    }
    let error                           =                                       self.output.flush();
    if !error.is_ok()
    {
      events.send
      (
        Event::new
        (
          EventType::Error("cannot flush to tty"),
          display,                      0,
          0,                            0,
          MouseButton::None,
        )
      ).unwrap();
    }
    let input                           =                                       self.input.clone();
    let ( sender, receiver ): ( Sender<bool>, Receiver<bool> )
                                        =                                       channel();
    self.messages                       =                                       Some(sender);
    self.listener
    = Some
      (
        thread::spawn
        (
          move ||
          {
            if let Ok(mut input) = input.lock()
            {
              let mut input             =                                       input.by_ref().bytes();
              let mut state: TTYState   =                                       TTYState::ExpectByte;
              let mut mouseState        =                                       MouseButton::None;
              let mut listOfParameters: Vec<usize>
                                        =                                       vec!();
              let mut currentParameter: usize
                                        =                                       0;
              let mut parameterPrefix: TTYPrefix
                                        =                                       TTYPrefix::None;
              'recvLoop:
                loop
                {
                  for event             in                                      receiver.try_iter()
                  {
                    if event
                    {
                      break 'recvLoop;
                    }
                  }
                  if let Some(byte) = input.next()
                  {
                    if let Ok(byte) = byte
                    {
                      state.nextState
                      (
                        byte,
                        Some(&focus),
                        display,
                        Some(&events),
                        Some(&mapOfFrames),
                        sizeX,          sizeY,
                        &mut mouseState,
                        &mut listOfParameters,
                        &mut currentParameter,
                        &mut parameterPrefix,
                      );
                    }
                  }
                }
            }
          }
        )
      );
  }

  pub fn turnOff
  (
    &mut self,
    events:                             &EventSender,
    display:                            DisplayID,
  )
  {
    let error
    = write!
      (
        self.output,
        "{}c",
        TTY_ESC,
      );
    if !error.is_ok()
    {
      events.send
      (
        Event::new
        (
          EventType::Error("cannot send to tty"),
          display,                      0,
          0,                            0,
          MouseButton::None,
        )
      ).unwrap();
    }
    let error                           =                                       self.output.flush();
    if !error.is_ok()
    {
      events.send
      (
        Event::new
        (
          EventType::Error("cannot flush to tty"),
          display,                      0,
          0,                            0,
          MouseButton::None,
        )
      ).unwrap();
    }
    if let Some(ref messages) = self.messages
    {
      messages.send(true).unwrap();
    }
    let listener: Option<JoinHandle<()>>
                                        =                                       self.listener.take();
    if let Some(listener) = listener
    {
      listener.join().unwrap();
    }
    unsafe
    {
      if let Ok(mut input) = self.input.lock()
      {
        let fd                          =                                       input.by_ref().as_raw_fd();
        if libc::fcntl( fd, libc::F_SETFL, self.fcntl ) < 0
        {
          events.send
          (
            Event::new
            (
              EventType::Error("cannot reset input to blocking"),
              display,                  0,
              0,                        0,
              MouseButton::None,
            )
          ).unwrap();
        }
      }
      if libc::tcsetattr( self.output.as_raw_fd(), libc::TCSAFLUSH, &mut self.termios) < 0
      {
        events.send
        (
          Event::new
          (
            EventType::Error("cannot enter cooked mode"),
            display,                    0,
            0,                          0,
            MouseButton::None,
          )
        ).unwrap();
      }
    }
  }

  pub fn flush
  (
    &mut self,
    events:                             EventSender,
    display:                            DisplayID,
  )
  {
    let error                           =                                       self.output.flush();
    if !error.is_ok()
    {
      events.send
      (
        Event::new
        (
          EventType::Error("cannot flush to tty"),
          display,                      0,
          0,                            0,
          MouseButton::None,
        )
      ).unwrap();
    }
  }

  #[allow(unused_variables)]
  pub fn drawStatusFrame
  (
    &mut self,
    this:                               &StatusFrame,
    events:                             &EventSender,
    lenX:                               usize,
    lenY:                               usize,
    minX:                               usize,
    minY:                               usize,
    maxX:                               usize,
    maxY:                               usize,
    cutX:                               usize,
    cutY:                               usize,
  )
  {
    let empty                           =                                       this.bgChar.to_string().repeat( lenX );
    write!
    (
      self.output,
      "{}{};{}H{}",
      TTY_CSI,
      ( minY + 1 ) as u16,              ( minX + 1 ) as u16,
      empty,
    ).unwrap();
    let mut offs:                 isize =                                       this.offs;
    let mut shift:                usize =                                       0;
    if offs < 0
    {
      shift                             =                                       -offs as usize;
      offs                              =                                       0;
    }
    if shift < lenX
    {
      let offs:                   usize =                                       offs as usize + cutX;
      write!
      (
        self.output,
        "{}{};{}H",
        TTY_CSI,
        ( minY + 1          ) as u16,   ( minX + 1  + shift ) as u16,
      ).unwrap();
      for char                          in                                      this.text.chars().skip( offs ).take( lenX )
      {
        if char == '\x1b'
        {
          break;
        }
        write!
        (
          self.output,
          "{}",
          char,
        ).unwrap();
      }
    }
  }

  #[allow(unused_variables)]
  pub fn drawTextFrame
  (
    &mut self,
    this:                               &TextFrame,
    events:                             &EventSender,
    lenX:                               usize,
    lenY:                               usize,
    minX:                               usize,
    minY:                               usize,
    maxX:                               usize,
    maxY:                               usize,
    cutX:                               usize,
    cutY:                               usize,
  )
  {
    let empty                           =                                       this.bgChar.to_string().repeat( lenX );
    {
      let posX: u16                     =                                       ( minX + 1 ) as u16;
      for posY                          in                                      ( minY + 1 ) as u16 .. ( maxY + 1 ) as u16
      {
        write!
        (
          self.output,
          "{}{};{}H{}",
          TTY_CSI,
          posY,                         posX,
          empty,
        ).unwrap();
      }
    }

    let mut offsX:  isize               =                                       this.offsX;
    let mut shiftX: usize               =                                       0;
    if offsX < 0
    {
      shiftX                            =                                       -offsX as usize;
      offsX                             =                                       0;
    }
    if shiftX < lenX
    {
      let offsX:      usize             =                                       offsX as usize + cutX;
      let mut offsY:  isize             =                                       this.offsY;
      let mut shiftY: usize             =                                       0;
      if offsY < 0
      {
        shiftY                          =                                       -offsY as usize;
        offsY                           =                                       0;
      }
      if shiftY < lenY
      {
        let offsY:                usize =                                       offsY as usize + cutY;
        let posX:                 usize =                                       minX + shiftX + 1;
        let posY:                 usize =                                       minY + shiftY + 1;
        for ( index, line )             in                                      this.lines.iter().skip( offsY ).take( lenY ).enumerate()
        {
          write!
          (
            self.output,
            "{}{};{}H",
            TTY_CSI,
            ( posY + index ) as u16,    ( posX         ) as u16,
          ).unwrap();
          for char                      in                                      line.chars().skip( offsX ).take( lenX )
          {
            if char == '\x1b'
            {
              break;
            }
            write!
            (
              self.output,
              "{}",
              char,
            ).unwrap();
          }
        }
      }
    }
  }

  #[allow(unused_variables)]
  pub fn drawEditorFrame
  (
    &mut self,
    this:                               &EditorFrame,
    events:                             &EventSender,
    lenX:                               usize,
    lenY:                               usize,
    minX:                               usize,
    minY:                               usize,
    maxX:                               usize,
    maxY:                               usize,
    cutX:                               usize,
    cutY:                               usize,
  )
  {
    let empty                           =                                       this.bgChar.to_string().repeat( lenX );
    {
      let posX:                     u16 =                                       ( minX + 1 ) as u16;
      for posY                          in                                      ( minY + 1 ) as u16 .. ( maxY + 1 ) as u16
      {
        write!
        (
          self.output,
          "{}{};{}H{}",
          TTY_CSI,
          posY,                         posX,
          empty,
        ).unwrap();
      }
    }

    let mut offsX:                isize =                                       this.offsX;
    let mut shiftX:               usize =                                       0;
    if offsX < 0
    {
      shiftX                            =                                       -offsX as usize;
      offsX                             =                                       0;
    }
    if shiftX < lenX
    {
      let offsX:                  usize =                                       offsX as usize + cutX;
      let mut offsY:              isize =                                       this.offsY;
      let mut shiftY:             usize =                                       0;
      if offsY < 0
      {
        shiftY                          =                                       -offsY as usize;
        offsY                           =                                       0;
      }
      if shiftY < lenY
      {
        let offsY:                usize =                                       offsY as usize + cutY;
        let posX:                 usize =                                       minX + shiftX + 1;
        let posY:                 usize =                                       minY + shiftY + 1;
        for ( index, words )            in                                      this.lines.iter().skip( offsY ).take( lenY ).enumerate()
        {
          write!
          (
            self.output,
            "{}{};{}H",
            TTY_CSI,
            ( posY + index ) as u16,    ( posX         ) as u16,
          ).unwrap();
          for word                      in                                      words
          {
            //most of them will be ignored by most of the terminals :'(
            let mut style               =                                       "".to_string();
            if ( word.font > 0 ) && ( word.font < 10 )                          { style = format!("{};", word.font + 10 ) }
            if ( word.flags & Style::Italic          ) != Style::None                     { style.push_str( "3;") }
            if ( word.flags & Style::Underline       ) != Style::None           { style.push_str( "4;") }
            if ( word.flags & Style::SlowBlink       ) != Style::None           { style.push_str( "5;") }
            if ( word.flags & Style::RapidBlink      ) != Style::None           { style.push_str( "6;") }
            if ( word.flags & Style::Inverse         ) != Style::None           { style.push_str( "7;") }
            if ( word.flags & Style::Conceal         ) != Style::None           { style.push_str( "8;") }
            if ( word.flags & Style::CrossedOut      ) != Style::None           { style.push_str( "9;") }
            if ( word.flags & Style::Fraktur         ) != Style::None           { style.push_str("20;") }
            if ( word.flags & Style::DoubleUnderline ) != Style::None           { style.push_str("21;") }
            if ( word.flags & Style::Framed          ) != Style::None           { style.push_str("51;") }
            if ( word.flags & Style::Encircled       ) != Style::None           { style.push_str("52;") }
            if ( word.flags & Style::Overlined       ) != Style::None           { style.push_str("53;") }
            let fgColour
            = match word.fgColour
              {
                Colour::RGB( red, green, blue )                                 => { format!( "38;2;{};{};{}", red, green, blue) },
                Colour::Standard( colour )                      if colour < 8
                                                                                => { format!( "38;5;{}", colour ) },
                Colour::Bright( colour )                        if colour < 8
                                                                                => { format!( "38;5;{}", colour + 8 ) },
                Colour::Cube( red, green, blue )                if red    < 6
                                                                && green  < 6
                                                                && blue   < 6
                                                                                => { format!( "38;5;{}", 16 + 36 * red + 6 * green + blue) },
                Colour::Grey( colour )                          if colour < 24
                                                                                => { format!( "38;5;{}", colour + 232 ) },
                Colour::Default                                                 => {   "39".to_string() },
                Colour::Black                                                   => {   "30".to_string() },
                Colour::Red                                                     => {   "31".to_string() },
                Colour::Green                                                   => {   "32".to_string() },
                Colour::Brown                                                   => {   "33".to_string() },
                Colour::Blue                                                    => {   "34".to_string() },
                Colour::Purple                                                  => {   "35".to_string() },
                Colour::Cyan                                                    => {   "36".to_string() },
                Colour::LightGrey                                               => {   "37".to_string() },
                Colour::DarkGrey                                                => { "1;30".to_string() },
                Colour::LightRed                                                => { "1;31".to_string() },
                Colour::LightGreen                                              => { "1;32".to_string() },
                Colour::Yellow                                                  => { "1;33".to_string() },
                Colour::LightBlue                                               => { "1;34".to_string() },
                Colour::LightPurple                                             => { "1;35".to_string() },
                Colour::LightCyan                                               => { "1;36".to_string() },
                Colour::White                                                   => { "1;37".to_string() },
                Colour::FaintBlack                                              => { "2;30".to_string() },
                Colour::FaintRed                                                => { "2;31".to_string() },
                Colour::FaintGreen                                              => { "2;32".to_string() },
                Colour::FaintYellow                                             => { "2;33".to_string() },
                Colour::FaintBlue                                               => { "2;34".to_string() },
                Colour::FaintPurple                                             => { "2;35".to_string() },
                Colour::FaintCyan                                               => { "2;36".to_string() },
                Colour::FaintWhite                                              => { "2;37".to_string() },
                Colour::BrightBlack                                             => {   "90".to_string() },
                Colour::BrightRed                                               => {   "91".to_string() },
                Colour::BrightGreen                                             => {   "92".to_string() },
                Colour::BrightYellow                                            => {   "93".to_string() },
                Colour::BrightBlue                                              => {   "94".to_string() },
                Colour::BrightPurple                                            => {   "95".to_string() },
                Colour::BrightCyan                                              => {   "96".to_string() },
                Colour::BrightWhite                                             => {   "97".to_string() },
                _                                                               => {     "".to_string() },
              };
            let bgColour
            = match word.bgColour
              {
                Colour::RGB( red, green, blue )                                 => { format!( "48;2;{};{};{}", red, green, blue ) },
                Colour::Standard( colour )                      if colour < 8
                                                                                => { format!( "48;5;{}", colour ) },
                Colour::Bright( colour )                        if colour < 8
                                                                                => { format!( "48;5;{}", colour + 8 ) },
                Colour::Cube( red, green, blue )                if red    < 6
                                                                && green  < 6
                                                                && blue   < 6
                                                                                => { format!( "48;5;{}", 16 + 36 * red + 6 * green + blue) },
                Colour::Grey( colour )                          if colour < 24
                                                                                => { format!( "48;5;{}", colour + 232 ) },
                Colour::Default                                                 => {   "49".to_string() },
                Colour::Black                                                   => {   "40".to_string() },
                Colour::Red                                                     => {   "41".to_string() },
                Colour::Green                                                   => {   "42".to_string() },
                Colour::Brown                                                   => {   "43".to_string() },
                Colour::Blue                                                    => {   "44".to_string() },
                Colour::Purple                                                  => {   "45".to_string() },
                Colour::Cyan                                                    => {   "46".to_string() },
                Colour::LightGrey                                               => {   "47".to_string() },
                Colour::DarkGrey                                                => { "1;40".to_string() },
                Colour::LightRed                                                => { "1;41".to_string() },
                Colour::LightGreen                                              => { "1;42".to_string() },
                Colour::Yellow                                                  => { "1;43".to_string() },
                Colour::LightBlue                                               => { "1;44".to_string() },
                Colour::LightPurple                                             => { "1;45".to_string() },
                Colour::LightCyan                                               => { "1;46".to_string() },
                Colour::White                                                   => { "1;47".to_string() },
                Colour::FaintBlack                                              => { "2;40".to_string() },
                Colour::FaintRed                                                => { "2;41".to_string() },
                Colour::FaintGreen                                              => { "2;42".to_string() },
                Colour::FaintYellow                                             => { "2;43".to_string() },
                Colour::FaintBlue                                               => { "2;44".to_string() },
                Colour::FaintPurple                                             => { "2;45".to_string() },
                Colour::FaintCyan                                               => { "2;46".to_string() },
                Colour::FaintWhite                                              => { "2;47".to_string() },
                Colour::BrightBlack                                             => {  "100".to_string() },
                Colour::BrightRed                                               => {  "101".to_string() },
                Colour::BrightGreen                                             => {  "102".to_string() },
                Colour::BrightYellow                                            => {  "103".to_string() },
                Colour::BrightBlue                                              => {  "104".to_string() },
                Colour::BrightPurple                                            => {  "105".to_string() },
                Colour::BrightCyan                                              => {  "106".to_string() },
                Colour::BrightWhite                                             => {  "107".to_string() },
                _                                                               => {     "".to_string() },
              };
            write!
            (
              self.output,
              "\x1b[{}{};{}m",
              style,
              fgColour,                 bgColour,
            ).unwrap();
            for char                    in                                      word.word.chars().skip( offsX ).take( lenX )
            {
              if char == '\x1b'
              {
                break;
              }
              write!
              (
                self.output,
                "{}",
                char,
              ).unwrap();
            }
            write!
            (
              self.output,
              "\x1b[0m",
            ).unwrap();
          }
        }
      }
    }
  }

  #[allow(unused_variables)]
  pub fn drawPixelFrame
  (
    &mut self,
    this:                               &PixelFrame,
    events:                             &EventSender,
    lenX:                               usize,
    lenY:                               usize,
    minX:                               usize,
    minY:                               usize,
    maxX:                               usize,
    maxY:                               usize,
    cutX:                               usize,
    cutY:                               usize,
  )
  {
  }

  #[allow(unused_variables)]
  pub fn drawPlotFrame
  (
    &mut self,
    this:                               &PlotFrame,
    events:                             &EventSender,
    lenX:                               usize,
    lenY:                               usize,
    minX:                               usize,
    minY:                               usize,
    maxX:                               usize,
    maxY:                               usize,
    cutX:                               usize,
    cutY:                               usize,
  )
  {
  }
}

impl TTYState
{
  pub fn nextState
  (
    &mut self,
    byte:                               u8,
    focus:                              Option<&Arc<Mutex<Box<FrameID>>>>,
    display:                            DisplayID,
    events:                             Option<&EventSender>,
    mapOfFrames:                        Option<&Arc<Mutex<Option<Box<[FrameID]>>>>>,
    width:                              usize,
    height:                             usize,
    mouseState:                         &mut MouseButton,
    listOfParameters:                   &mut Vec<usize>,
    currentParameter:                   &mut usize,
    parameterPrefix:                    &mut TTYPrefix,
  ) -> Option<Event>
  {
    let mut returnValue: Option<Event>  =                                       None;
    match self
    {
      TTYState::ExpectByte                                                      =>
      {
        match byte
        {
          0x1b                                                                  =>
          {
            *self                       =                                       TTYState::Escape;
          },
          byte @ 0x20...0x7e                                                    =>
          {
            //println!("char: {}", byte as char);
            let mut frame: FrameID      =                                       0;
            if let Some(focus) = focus
            {
              if let Ok(focus) = focus.lock()
              {
                frame                   =                                       **focus;
              }
            }
            let newEvent: Event
            = Event::new
              (
                EventType::Character(byte as char),
                display,                frame,
                0,                      0,
                *mouseState,
              );
            if let Some(events) = events
            {
              events.send(newEvent).unwrap();
            }
            else
            {
              returnValue                =                                       Some(newEvent);
            }
          },
          _                                                                     =>
          {
          },
        }
      },
      TTYState::Escape                                                          =>
      {
        match byte
        {
          r @ 0x30 ... 0x39                                                     =>
          {
            *parameterPrefix            =                                       TTYPrefix::Escape;
            *listOfParameters           =                                       vec!();
            *currentParameter           =                                       r as usize - 0x30;
            *self                       =                                       TTYState::ParseArgument;
          },
          0x5b                                                                  =>
          {
            *self                       =                                       TTYState::CSI;
          },
          c @ _                                                                 =>
          {
            let mut frame: FrameID      =                                       0;
            if let Some(focus) = focus
            {
              if let Ok(focus) = focus.lock()
              {
                frame                   =                                       **focus;
              }
            }
            let newEvent: Event
            = Event::new
              (
                EventType::Escape,
                display,                frame,
                0,                      0,
                *mouseState,
              );
            if let Some(events) = events
            {
              events.send(newEvent).unwrap();
            }
            else
            {
              returnValue               =                                       Some(newEvent);
            }
            if c != 0x1b
            {
              *self                     =                                       TTYState::ExpectByte;
            }
          }
        }
      },
      TTYState::CSI                                                             =>
      {
        match byte
        {
          r @ 0x30 ... 0x39                                                     =>
          {
            *parameterPrefix            =                                       TTYPrefix::CSI;
            *listOfParameters           =                                       vec!();
            *currentParameter           =                                       r as usize - 0x30;
            *self                       =                                       TTYState::ParseArgument;
          },
          0x3c                                                                  =>
          {
            *parameterPrefix            =                                       TTYPrefix::Mouse;
            *listOfParameters           =                                       vec!();
            *currentParameter           =                                       0;
            *self                       =                                       TTYState::ParseArgument;
          },
          _                                                                     =>
          {
            *self                       =                                       TTYState::ExpectByte;
          }
        }
      },
      TTYState::ParseArgument                                                   =>
      {
        match byte
        {
          r @ 0x30 ... 0x39                                                     =>
          {
            *currentParameter           =                                       *currentParameter * 10 + r as usize - 0x30;
          },
          0x3b                                                                  =>
          {
            //println!("newArgument: {}", *currentParameter);
            listOfParameters.push(*currentParameter);
            *currentParameter           =                                       0;
          },
          0x4d  if listOfParameters.len() == 2
                && *parameterPrefix == TTYPrefix::Mouse                         =>
          {
            listOfParameters.push(*currentParameter);
            let frame: FrameID
            = if let Some(mapOfFrames) = mapOfFrames                            //did I get a reference to a map of frames?
              {
                if let Ok(mapOfFrames) = mapOfFrames.lock()                     //can I access it?
                {
                  if let Some(ref mapOfFrames) = *mapOfFrames                   //is it not empty?
                  {
                    let x               =                                       listOfParameters[1] - 1;
                    let y               =                                       listOfParameters[2] - 1;
                    if ( x < width  )
                    && ( y < height )
                    {
                      mapOfFrames[ y * width + x ]
                    }
                    else
                    {
                      0
                    }
                  }
                  else
                  {
                    0
                  }
                }
                else
                {
                  0
                }
              }
              else
              {
                0
              };
            let mut theEvent: Option<EventType>
                                        =                                       None;
            match listOfParameters[0]
            {
              0                                                                 =>
              {
                *mouseState             |=                                      MouseButton::LeftDown;
                theEvent                =                                       Some(EventType::MouseLeftButtonPressed);
              },
              1                                                                 =>
              {
                *mouseState             |=                                      MouseButton::MiddleDown;
                theEvent                =                                       Some(EventType::MouseMiddleButtonPressed);
              },
              2                                                                 =>
              {
                *mouseState             |=                                      MouseButton::RightDown;
                theEvent                =                                       Some(EventType::MouseRightButtonPressed);
              },
              32                                                                =>
              {
                theEvent                =                                       Some(EventType::MouseMoveWithLeftButton);
              },
              33                                                                =>
              {
                theEvent                =                                       Some(EventType::MouseMoveWithMiddleButton);
              },
              34                                                                =>
              {
                theEvent                =                                       Some(EventType::MouseMoveWithRightButton);
              },
              35                                                                =>
              {
                theEvent                =                                       Some(EventType::MouseOver);
              }
              64                                                                =>
              {
                theEvent                =                                       Some(EventType::MouseWheelUp);
              },
              65                                                                =>
              {
                theEvent                =                                       Some(EventType::MouseWheelDown);
              },
              _                                                                 => { /* invalid */ },
            }
            *self                       =                                       TTYState::ExpectByte;
            if let Some(theEvent) = theEvent
            {
              let newEvent: Event
              = Event::new
                (
                  theEvent,
                  display,              frame,
                  listOfParameters[2] - 1,
                  listOfParameters[1] - 1,
                  *mouseState,
                );
              if let Some(events) = events
              {
                events.send(newEvent).unwrap();
              }
              else
              {
                returnValue             =                                       Some(newEvent);
              }
            }
          },
          0x6d  if listOfParameters.len() == 2
                && *parameterPrefix == TTYPrefix::Mouse                         =>
          {
            listOfParameters.push(*currentParameter);
            let frame: FrameID
            = if let Some(mapOfFrames) = mapOfFrames                            //did I get a reference to a map of frames?
              {
                if let Ok(mapOfFrames) = mapOfFrames.lock()                     //can I access it?
                {
                  if let Some(ref mapOfFrames) = *mapOfFrames                   //is it not empty?
                  {
                    let x               =                                       listOfParameters[1] - 1;
                    let y               =                                       listOfParameters[2] - 1;
                    if ( x < width  )
                    && ( y < height )
                    {
                      mapOfFrames[ y * width + x ]
                    }
                    else
                    {
                      0
                    }
                  }
                  else
                  {
                    0
                  }
                }
                else
                {
                  0
                }
              }
              else
              {
                0
              };
            let mut theEvent: Option<EventType>
                                        =                                       None;
            match listOfParameters[0]
            {
              0                                                                 =>
              {
                *mouseState             &=                                      !MouseButton::LeftDown;
                theEvent                =                                       Some(EventType::MouseLeftButtonReleased);
              },
              1                                                                 =>
              {
                *mouseState             &=                                      !MouseButton::MiddleDown;
                theEvent                =                                       Some(EventType::MouseMiddleButtonReleased);
              },
              2                                                                 =>
              {
                *mouseState             &=                                      !MouseButton::RightDown;
                theEvent                =                                       Some(EventType::MouseRightButtonReleased);
              },
              _                                                                 => { /* invalid */ },
            }
            *self                       =                                       TTYState::ExpectByte;
            if let Some(theEvent) = theEvent
            {
              let newEvent: Event
              = Event::new
                (
                  theEvent,
                  display,              frame,
                  listOfParameters[2] - 1,
                  listOfParameters[1] - 1,
                  *mouseState,
                );
              if let Some(events) = events
              {
                events.send(newEvent).unwrap();
              }
              else
              {
                returnValue             =                                       Some(newEvent);
              }
            }
          },
          0x52  if listOfParameters.len() == 1
                && *parameterPrefix == TTYPrefix::CSI                           =>
          {
            listOfParameters.push(*currentParameter);
            //println!("size: {}x{}", listOfParameters[1], listOfParameters[0]);
            *self                       =                                        TTYState::ExpectByte;
            let newEvent: Event
            = Event::new
              (
                EventType::CursorPosition,
                display,              0,
                listOfParameters[1] - 0,
                listOfParameters[0] - 0,
                *mouseState,
              );
            if let Some(events) = events
            {
              events.send(newEvent).unwrap();
            }
            else
            {
              returnValue               =                                       Some(newEvent);
            }
          },
          _r @ _                                                                =>
          {
           //println!("cannot parse {:?} {}", listOfParameters, r as char);
           *self                        =                                       TTYState::ExpectByte;
          },
        }
      },
    }
    returnValue
  }
}

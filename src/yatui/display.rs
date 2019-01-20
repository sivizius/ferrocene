use crate::yatui::*;
use crate::yatui::frame::*;
use crate::yatui::event::*;

use libc::
{
  termios
};
use std::
{
  io::
  {
    Bytes,
    Read,
    self,
    Write,
  },
  marker::
  {
    Send,
    Sync
  },
  os::
  {
    unix::
    {
      io::
      {
        AsRawFd,
        RawFd,
      },
    },
  },
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
    RwLock,
  },
  thread::
  {
    sleep,
    JoinHandle,
  },
  time::
  {
    Duration,
    SystemTime
  },
};

pub trait ReadableFd:   Read  + AsRawFd + Send  + Sync                        {}
pub trait WriteableFd:  Write + AsRawFd                                       {}
impl <Type:             Read  + AsRawFd + Send  + Sync> ReadableFd  for Type  {}
impl <Type:             Write + AsRawFd               > WriteableFd for Type  {}

pub type DisplayID                      =                                       usize;

pub const TTY_ESC:                 &str =                                       "\x1b";
pub const TTY_CSI:                 &str =                                       "\x1b[";
pub enum TTYState
{
  ExpectByte,
  Escape,
  CSI,
  ParseArgument,
}
pub struct TTYDisplay
{
  pub input:                            Arc<Mutex<Box<ReadableFd>>>,
  pub output:                           Box<WriteableFd>,
  listener:                             Option<JoinHandle<()>>,
  messages:                             Option<Sender<bool>>,
  termios:                              libc::termios,
  fcntl:                                i32,
}

pub enum DisplayType
{
  TTY(TTYDisplay),
}

pub const Display_None:           Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;
pub const Display_RealTime:       Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0001;

pub const Display_NeedRefresh:    Flags =                                       0b0100_0000_0000_0000_0000_0000_0000_0000;
pub const Display_NeedRemap:      Flags =                                       0b1000_0000_0000_0000_0000_0000_0000_0000;
pub const Display_MaskRefresh:    Flags =                                       Display_NeedRefresh | Display_NeedRemap;

pub struct Display
{
  pub flags:                            Flags,
  pub this:                             DisplayID,
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub sizeX:                            usize,
  pub sizeY:                            usize,
  pub cursorX:                          usize,
  pub cursorY:                          usize,
  mapOfFrames:                          Arc<Mutex<Box<[FrameID]>>>,
  pub mainFrame:                        FrameID,
  pub focusedFrame:                     Arc<Mutex<Box<FrameID>>>,
  pub lastRefresh:                      SystemTime,
  pub nextRefresh:                      Duration,
  pub display:                          DisplayType,
}

impl Display
{
  pub fn draw
  (
    &mut self,
    listOfFrames:                       &mut Vec<Option<Frame>>,
    events:                             &EventSender,
    drawFrame:                          FrameID,
    posX:                               isize,
    posY:                               isize,
    lenX:                               usize,
    lenY:                               usize,
  )
  {
    let mut minX: isize                 =                                       posX;
    let mut minY: isize                 =                                       posY;
    let mut maxX: isize                 =                                       posX + lenX as isize;
    let mut maxY: isize                 =                                       posY + lenY as isize;
    if ( drawFrame == 0 )
    || ( drawFrame > listOfFrames.len() )
    {
      // invalid frame to draw
    }
    else if ( minX > self.sizeX as isize )
         || ( minY > self.sizeY as isize )
    {
      // ignore
    }
    else if listOfFrames [ drawFrame - 1 ].is_some()
    {
      let mut refFrame                  =                                       listOfFrames [ drawFrame - 1 ].take();
      let mut cutX: usize               =                                       0;
      let mut cutY: usize               =                                       0;
      if minX < 0
      {
        cutX                            =                                       -minX as usize;
        minX                            =                                       0;
      }
      if minY < 0
      {
        cutY                            =                                       -minY as usize;
        minY                            =                                       0;
      }
      if maxX > self.sizeX as isize
      {
        maxX                            =                                       self.sizeX as isize;
      }
      if maxY > self.sizeY as isize
      {
        maxY                            =                                       self.sizeY as isize;
      }
      let minX: usize                   =                                       minX as usize;
      let minY: usize                   =                                       minY as usize;
      let maxX: usize                   =                                       maxX as usize;
      let maxY: usize                   =                                       maxY as usize;
      let lenX: usize                   =                                       ( maxX - minX )  as usize;
      let lenY: usize                   =                                       ( maxY - minY )  as usize;
      if ( self.flags & Display_NeedRemap ) != 0
      {
        if let Ok(mut mapOfFrames) = self.mapOfFrames.lock()
        {
          self.flags                    &=                                      !Display_NeedRemap;
          for y                         in                                      minY .. maxY
          {
            for x                       in                                      minX .. maxX
            {
              mapOfFrames [ x + y * self.sizeX ]
                                        =                                       drawFrame;
            }
          }
        }
      }
      match refFrame.as_mut().unwrap()
      {
        Frame::Status ( ref frame ) =>
        {
          match self.display
          {
            DisplayType::TTY(ref mut output) =>
            {
              let empty                 =                                       frame.bgChar.to_string().repeat( lenX );
              write!
              (
                output.output,
                "{}{};{}H{}",
                TTY_CSI,
                ( minY + 1 ) as u16,
                ( minX + 1 ) as u16,
                empty,
              ).unwrap();
              let mut offs:   isize     =                                       frame.offs;
              let mut shift:  usize     =                                       0;
              if offs < 0
              {
                shift                   =                                       -offs as usize;
                offs                    =                                       0;
              }
              if shift < lenX
              {
                let offs:     usize     =                                       offs as usize + cutX;
                write!
                (
                  output.output,
                  "{}{};{}H",
                  TTY_CSI,
                  ( minY + 1          ) as u16,
                  ( minX + 1  + shift ) as u16,
                ).unwrap();
                for char                in                                      frame.text.chars().skip( offs ).take( lenX )
                {
                  if char == '\x1b'
                  {
                    break;
                  }
                  write!
                  (
                    output.output,
                    "{}",
                    char,
                  ).unwrap();
                }
              }
            },
          }
        },
        Frame::Text ( ref frame ) =>
        {
          match self.display
          {
            DisplayType::TTY(ref mut output) =>
            {
              let empty                 =                                       frame.bgChar.to_string().repeat( lenX );
              {
                let posX: u16           =                                       ( minX + 1 ) as u16;
                for posY                in                                      ( minY + 1 ) as u16 .. ( maxY + 1 ) as u16
                {
                  write!
                  (
                    output.output,
                    "{}{};{}H{}",
                    TTY_CSI,
                    posY,
                    posX,
                    empty,
                  ).unwrap();
                }
              }
    
              let mut offsX:  isize     =                                       frame.offsX;
              let mut shiftX: usize     =                                       0;
              if offsX < 0
              {
                shiftX                  =                                       -offsX as usize;
                offsX                   =                                       0;
              }
              if shiftX < lenX
              {
                let offsX:      usize   =                                       offsX as usize + cutX;
                let mut offsY:  isize   =                                       frame.offsY;
                let mut shiftY: usize   =                                       0;
                if offsY < 0
                {
                  shiftY                =                                       -offsY as usize;
                  offsY                 =                                       0;
                }
                if shiftY < lenY
                {
                  let offsY:      usize =                                       offsY as usize + cutY;
                  let posX:       usize =                                       minX + shiftX + 1;
                  let posY:       usize =                                       minY + shiftY + 1;
                  for ( index, line )   in                                      frame.lines.iter().skip( offsY ).take( lenY ).enumerate()
                  {
                    write!
                    (
                      output.output,
                      "{}{};{}H",
                      TTY_CSI,
                      ( posY + index ) as u16,
                      ( posX         ) as u16, 
                    ).unwrap();
                    for char            in                                      line.chars().skip( offsX ).take( lenX )
                    {
                      if char == '\x1b'
                      {
                        break;
                      }
                      write!
                      (
                        output.output,
                        "{}",
                        char,
                      ).unwrap();
                    }
                  }
                }
              }
            },
          }
        },
        Frame::Editor ( ref frame ) =>
        {
          match self.display
          {
            DisplayType::TTY(ref mut output) =>
            {
              let empty                 =                                       frame.bgChar.to_string().repeat( lenX );
              {
                let posX: u16           =                                       ( minX + 1 ) as u16;
                for posY                in                                      ( minY + 1 ) as u16 .. ( maxY + 1 ) as u16
                {
                  write!
                  (
                    output.output,
                    "{}{};{}H{}",
                    TTY_CSI,
                    posY,
                    posX,
                    empty,
                  ).unwrap();
                }
              }
    
              let mut offsX:  isize     =                                       frame.offsX;
              let mut shiftX: usize     =                                       0;
              if offsX < 0
              {
                shiftX                  =                                       -offsX as usize;
                offsX                   =                                       0;
              }
              if shiftX < lenX
              {
                let offsX:      usize   =                                       offsX as usize + cutX;
                let mut offsY:  isize   =                                       frame.offsY;
                let mut shiftY: usize   =                                       0;
                if offsY < 0
                {
                  shiftY                =                                       -offsY as usize;
                  offsY                 =                                       0;
                }
                if shiftY < lenY
                {
                  let offsY:      usize =                                       offsY as usize + cutY;
                  let posX:       usize =                                       minX + shiftX + 1;
                  let posY:       usize =                                       minY + shiftY + 1;
                  for ( index, words )  in                                      frame.lines.iter().skip( offsY ).take( lenY ).enumerate()
                  {
                    write!
                    (
                      output.output,
                      "{}{};{}H",
                      TTY_CSI,
                      ( posY + index ) as u16,
                      ( posX         ) as u16, 
                    ).unwrap();
                    for word            in                                      words
                    {
                      let mut style     =                                       "".to_string();
                      if ( word.font > 0 ) && ( word.font < 10 )                { style = format!("{};", word.font + 10 ) }
                      if ( word.flags & Style_Italic          ) == 1            { style.push_str("3;") }
                      if ( word.flags & Style_Underline       ) == 1            { style.push_str("4;") }
                      if ( word.flags & Style_SlowBlink       ) == 1            { style.push_str("5;") }
                      if ( word.flags & Style_RapidBlink      ) == 1            { style.push_str("6;") }
                      if ( word.flags & Style_Inverse         ) == 1            { style.push_str("7;") }
                      if ( word.flags & Style_Conceal         ) == 1            { style.push_str("8;") }
                      if ( word.flags & Style_CrossedOut      ) == 1            { style.push_str("9;") }
                      if ( word.flags & Style_Fraktur         ) == 1            { style.push_str("20;") }
                      if ( word.flags & Style_DoubleUnderline ) == 1            { style.push_str("21;") }
                      if ( word.flags & Style_Framed          ) == 1            { style.push_str("51;") }
                      if ( word.flags & Style_Encircled       ) == 1            { style.push_str("52;") }
                      if ( word.flags & Style_Overlined       ) == 1            { style.push_str("53;") }
                      let fgColour
                      = match word.fgColour
                        {
                          Colour::Default                  => { "39".to_string() },
                          Colour::RGB( red, green, blue )  => { format!( "38;2;{};{};{}", red, green, blue) },
                          Colour::Standard( colour )       if colour < 8
                                                            => { format!( "38;5;{}", colour ) },
                          Colour::Bright( colour )         if colour < 8
                                                            => { format!( "38;5;{}", colour + 8 ) },
                          Colour::Cube( red, green, blue ) if red < 6
                                                            && green < 6
                                                            && blue < 6
                                                            => { format!( "38;5;{}", 16 + 36 * red + 6 * green + blue) },
                          Colour::Grey( colour )           if colour < 24
                                                            => { format!( "38;5;{}", colour + 232 ) },
                          Colour::Black                    => { "30".to_string() },
                          Colour::Red                      => { "31".to_string() },
                          Colour::Green                    => { "32".to_string() },
                          Colour::Brown                    => { "33".to_string() },
                          Colour::Blue                     => { "34".to_string() },
                          Colour::Purple                   => { "35".to_string() },
                          Colour::Cyan                     => { "36".to_string() },
                          Colour::LightGrey                => { "37".to_string() },
                          Colour::DarkGrey                 => { "1;30".to_string() },
                          Colour::LightRed                 => { "1;31".to_string() },
                          Colour::LightGreen               => { "1;32".to_string() },
                          Colour::Yellow                   => { "1;33".to_string() },
                          Colour::LightBlue                => { "1;34".to_string() },
                          Colour::LightPurple              => { "1;35".to_string() },
                          Colour::LightCyan                => { "1;36".to_string() },
                          Colour::White                    => { "1;37".to_string() },
                          Colour::FaintBlack               => { "2;30".to_string() },
                          Colour::FaintRed                 => { "2;31".to_string() },
                          Colour::FaintGreen               => { "2;32".to_string() },
                          Colour::FaintYellow              => { "2;33".to_string() },
                          Colour::FaintBlue                => { "2;34".to_string() },
                          Colour::FaintPurple              => { "2;35".to_string() },
                          Colour::FaintCyan                => { "2;36".to_string() },
                          Colour::FaintWhite               => { "2;37".to_string() },
                          Colour::BrightBlack              => { "90".to_string() },
                          Colour::BrightRed                => { "91".to_string() },
                          Colour::BrightGreen              => { "92".to_string() },
                          Colour::BrightYellow             => { "93".to_string() },
                          Colour::BrightBlue               => { "94".to_string() },
                          Colour::BrightPurple             => { "95".to_string() },
                          Colour::BrightCyan               => { "96".to_string() },
                          Colour::BrightWhite              => { "97".to_string() },
                          _                                 => { "".to_string()  },
                        };
                      let bgColour
                      = match word.bgColour
                        {
                          Colour::Default                  => { "49".to_string() },
                          Colour::RGB( red, green, blue )  => { format!( "48;2;{};{};{}", red, green, blue ) },
                          Colour::Standard( colour )       if colour < 8
                                                            => { format!( "48;5;{}", colour ) },
                          Colour::Bright( colour )         if colour < 8
                                                            => { format!( "48;5;{}", colour + 8 ) },
                          Colour::Cube( red, green, blue ) if red < 6
                                                            && green < 6
                                                            && blue < 6
                                                            => { format!( "48;5;{}", 16 + 36 * red + 6 * green + blue) },
                          Colour::Grey( colour )           if colour < 24
                                                            => { format!( "48;5;{}", colour + 232 ) },
                          Colour::Black                    => { "40".to_string() },
                          Colour::Red                      => { "41".to_string() },
                          Colour::Green                    => { "42".to_string() },
                          Colour::Brown                    => { "43".to_string() },
                          Colour::Blue                     => { "44".to_string() },
                          Colour::Purple                   => { "45".to_string() },
                          Colour::Cyan                     => { "46".to_string() },
                          Colour::LightGrey                => { "47".to_string() },
                          Colour::DarkGrey                 => { "1;40".to_string() },
                          Colour::LightRed                 => { "1;41".to_string() },
                          Colour::LightGreen               => { "1;42".to_string() },
                          Colour::Yellow                   => { "1;43".to_string() },
                          Colour::LightBlue                => { "1;44".to_string() },
                          Colour::LightPurple              => { "1;45".to_string() },
                          Colour::LightCyan                => { "1;46".to_string() },
                          Colour::White                    => { "1;47".to_string() },
                          Colour::FaintBlack               => { "2;40".to_string() },
                          Colour::FaintRed                 => { "2;41".to_string() },
                          Colour::FaintGreen               => { "2;42".to_string() },
                          Colour::FaintYellow              => { "2;43".to_string() },
                          Colour::FaintBlue                => { "2;44".to_string() },
                          Colour::FaintPurple              => { "2;45".to_string() },
                          Colour::FaintCyan                => { "2;46".to_string() },
                          Colour::FaintWhite               => { "2;47".to_string() },
                          Colour::BrightBlack              => { "100".to_string() },
                          Colour::BrightRed                => { "101".to_string() },
                          Colour::BrightGreen              => { "102".to_string() },
                          Colour::BrightYellow             => { "103".to_string() },
                          Colour::BrightBlue               => { "104".to_string() },
                          Colour::BrightPurple             => { "105".to_string() },
                          Colour::BrightCyan               => { "106".to_string() },
                          Colour::BrightWhite              => { "107".to_string() },
                          _                                 => { "".to_string()  },
                        };
                      write!
                      (
                        output.output,
                        "\x1b[{}{};{}m",
                        style,
                        fgColour,
                        bgColour,
                      ).unwrap();
                      for char          in                                      word.word.chars().skip( offsX ).take( lenX )
                      {
                        if char == '\x1b'
                        {
                          break;
                        }
                        write!
                        (
                          output.output,
                          "{}",
                          char,
                        ).unwrap();
                      }
                      write!
                      (
                        output.output,
                        "\x1b[0m",
                      ).unwrap();
                    }
                  }
                }
              }
            },
          }
        },
        Frame::Pixel(ref frame) =>
        {
          //TODO
          match self.display
          {
            DisplayType::TTY(ref mut output) =>
            {
            },
          }
        },
        Frame::Plot( ref frame ) =>
        {
          //TODO
          match self.display
          {
            DisplayType::TTY(ref mut output) =>
            {
            },
          }
        },
        Frame::Parent ( ref mut frame ) =>
        {
          match frame.typeOfTiling
          {
            Tiling::None                =>                                      {},
            Tiling::Grid                =>
            {
              let countX                =                                       frame.gridBordersX.len() - 1;
              let countY                =                                       frame.gridBordersY.len() - 1;
              for mut instance          in                                      &mut frame.listOfInstances
              {
                if ( instance.gridOriginX <= countX )
                && ( instance.gridOriginY <= countY )
                && ( instance.gridLenghtX > 0 )
                && ( instance.gridLenghtY > 0 )
                {
                  if instance.gridOriginX + instance.gridLenghtX > countX
                  {
                    instance.gridLenghtX
                                        =                                       countX - instance.gridOriginX;
                  }
                  if instance.gridOriginY + instance.gridLenghtY > countY
                  {
                    instance.gridLenghtY
                                        =                                       countY - instance.gridOriginY;
                  }
                  instance.posX         =                                       frame.gridBordersX [ instance.gridOriginX ];
                  instance.posY         =                                       frame.gridBordersY [ instance.gridOriginY ];
                  instance.lenX         =                                       ( frame.gridBordersX [ instance.gridOriginX + instance.gridLenghtX ] - instance.posX ) as usize;
                  instance.lenY         =                                       ( frame.gridBordersY [ instance.gridOriginY + instance.gridLenghtY ] - instance.posY ) as usize;
                }
              }
            }
          }
          for instance                  in                                      &mut frame.listOfInstances
          {
            let posX                    =                                       posX + instance.posX;
            let posY                    =                                       posY + instance.posY;
            let lenX                    =                                       instance.lenX;
            let lenY                    =                                       instance.lenY;
            let next                    =                                       instance.frame;
            self.draw
            (
              listOfFrames,
              events,
              next,
              posX,                     posY,
              lenX,                     lenY,
            );
          }
        },
        Frame::Layers( ref frame ) =>
        {
          for layer                     in                                      &frame.listOfLayers
          {
            self.draw
            (
              listOfFrames,
              events,
              *layer,
              posX,                     posY,
              lenX,                     lenY,
            );
          }
        },
      }
      listOfFrames [ drawFrame - 1 ]    =                                       refFrame.take();
    }
    else
    {
      //wut
    }
  }
  pub fn turnOn
  (
    &mut self,
    events:                             EventSender,
  )
  {
    match self.display
    {
      DisplayType::TTY(ref mut output) =>
      {
        unsafe
        {
          let mut termios               =                                       output.termios.clone();
          libc::cfmakeraw(&mut termios);
          if libc::tcsetattr( output.output.as_raw_fd(), libc::TCSAFLUSH, &mut termios) < 0
          {
            events.send
            (
              event::Event::new
              (
                EventType::Error("cannot enter raw mode"),
                self.this,              0,
                0,                      0,
                0,
              )
            ).unwrap();
          }
          if let Ok(mut input) = output.input.lock()
          {
            let fd                      =                                       input.by_ref().as_raw_fd();
            output.fcntl                =                                       libc::fcntl(fd, libc::F_GETFL);
            if libc::fcntl( fd, libc::F_SETFL, output.fcntl | libc::O_NONBLOCK) < 0
            {
              events.send
              (
                event::Event::new
                (
                  EventType::Error("cannot make input non-blocking"),
                  self.this,              0,
                  0,                      0,
                  0,
                )
              ).unwrap();
            }
          }
        }
        let error
        = write!
          (
            output.output,
            "{}]0;{}\x07{}{}J{}{};{}H{}?{}l{}?{}h{}?{}h",
            TTY_ESC,                    "Hello World",
            TTY_CSI,                    2,
            TTY_CSI,                    1, 1,
            TTY_CSI,                    25,
            TTY_CSI,                    1003,
            TTY_CSI,                    1006,
          );
        if !error.is_ok()
        {
          events.send
          (
            event::Event::new
            (
              EventType::Error("cannot send to tty"),
              self.this,                0,
              0,                        0,
              0,
            )
          ).unwrap();
        }
        let error                       =                                       output.output.flush();
        if !error.is_ok()
        {
          events.send
          (
            event::Event::new
            (
              EventType::Error("cannot flush to tty"),
              self.this,                0,
              0,                        0,
              0
            )
          ).unwrap();
        }
        let input                       =                                       output.input.clone();
        let mapOfFrames                 =                                       self.mapOfFrames.clone();
        let focus                       =                                       self.focusedFrame.clone();
        let width                       =                                       self.sizeX;
        let display                     =                                       self.this;
        let ( sender, receiver ): ( Sender<bool>, Receiver<bool> )
                                        =                                       channel();
        output.messages                 =                                       Some(sender);
        output.listener
        = Some
          (
            thread::spawn
            (
              move ||
              {
                if let Ok(mut input) = input.lock()
                {
                  let mut input         =                                       input.by_ref().bytes();
                  let mut state: TTYState
                                        =                                       TTYState::ExpectByte;
                  let mut mouseState: Flags
                                        =                                       MouseButton_None;
                  let mut listOfParameters: Vec<usize>
                                        =                                       vec!();
                  let mut currentParameter: usize
                                        =                                       0;
                  'recvLoop:
                    loop
                    {
                      for event         in                                      receiver.try_iter()
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
                            width,
                            &mut mouseState,
                            &mut listOfParameters,
                            &mut currentParameter,
                          );
                        }
                      }
                    }
                }
              }
            )
          );
      }
    }
  }

  pub fn turnOff
  (
    &mut self,
    events:                             EventSender,
  )
  {
    match self.display
    {
      DisplayType::TTY(ref mut output) =>
      {
        let error
        = write!
          (
            output.output,
            "{}c",
            TTY_ESC,
          );
        if !error.is_ok()
        {
          events.send
          (
            event::Event::new
            (
              EventType::Error("cannot send to tty"),
              self.this,                0,
              0,                        0,
              0,
            )
          ).unwrap();
        }
        let error                       =                                       output.output.flush();
        if !error.is_ok()
        {
          events.send
          (
            event::Event::new
            (
              EventType::Error("cannot flush to tty"),
              self.this,                0,
              0,                        0,
              0,
            )
          ).unwrap();
        }
        if let Some(ref messages) = output.messages
        {
          messages.send(true).unwrap();
        }
        let listener: Option<JoinHandle<()>>
                                        =                                       output.listener.take();
        if let Some(listener) = listener
        {
          listener.join().unwrap();
        }
        unsafe
        {
          if let Ok(mut input) = output.input.lock()
          {
            let fd                      =                                       input.by_ref().as_raw_fd();
            if libc::fcntl( fd, libc::F_SETFL, output.fcntl ) < 0
            {
              events.send
              (
                event::Event::new
                (
                  EventType::Error("cannot reset input to blocking"),
                  self.this,              0,
                  0,                      0,
                  0,
                )
              ).unwrap();
            }
          }
          if libc::tcsetattr( output.output.as_raw_fd(), libc::TCSAFLUSH, &mut output.termios) < 0
          {
            events.send
            (
              event::Event::new
              (
                EventType::Error("cannot enter cannoncial mode"),
                self.this,              0,
                0,                      0,
                0,
              )
            ).unwrap();
          }
        }
      }
    }
  }
}

impl TTYDisplay
{
  pub fn new
  (
    flags:                              Flags,
    offsX:                              isize,
    offsY:                              isize,
    cursorX:                            usize,
    cursorY:                            usize,
    mut input:                          Box<ReadableFd>,
    mut output:                         Box<WriteableFd>,
    refreshRate:                        u64,
  ) -> Result<Display, &'static str>
  {
    let ( sizeX, sizeY, resX, resY )    =                                       Self::getTerminalSize ( &mut input, &mut output )?;
    let mut mapOfFrames: Vec<FrameID>   =                                       Vec::with_capacity( ( sizeX * sizeY ) as usize );
    mapOfFrames.resize(( sizeX * sizeY ) as usize, 0);
    let mapOfFrames: Box<[FrameID]>     =                                       mapOfFrames.into_boxed_slice();
    unsafe
    {
      let mut termios: libc::termios    =                                       mem::zeroed();
      if libc::tcgetattr( output.as_raw_fd(), &mut termios) < 0
      {
        Err("cannot get termios structure")
      }
      else
      {
        Ok
        (
          Display
          {
            flags:                            flags | Display_NeedRefresh | Display_NeedRemap,
            this:                             0,
            offsX:                            offsX,
            offsY:                            offsY,
            sizeX:                            sizeX as usize,
            sizeY:                            sizeY as usize,
            cursorX:                          cursorX,
            cursorY:                          cursorY,
            mapOfFrames:                      Arc::new(Mutex::new(mapOfFrames)),
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
  ) -> Result<( u16, u16, u16, u16 ), &'static str>
  {
    unsafe
    {
      let mut winsize: libc::winsize    =                                       mem::zeroed();
      if libc::ioctl( output.as_raw_fd(), libc::TIOCGWINSZ, &mut winsize as *mut _) < 0
      || true
      {
        write!
        (
          output,
          "{}999;999H{}6n\n",
          TTY_CSI,
          TTY_CSI,
        ).unwrap();
        let mut state: TTYState         =                                       TTYState::ExpectByte;
        let mut mouseState              =                                       0;
        let mut listOfParameters        =                                       vec!();
        let mut currentParameter        =                                       0;
        let mut returnValue: Option<Event>
                                        =                                       None;
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
                0,
                &mut mouseState,
                &mut listOfParameters,
                &mut currentParameter,
              );
            }
          }
        }
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
      else
      {
        println!("pixel: {}x{}", winsize.ws_xpixel, winsize.ws_ypixel);
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
    mapOfFrames:                        Option<&Arc<Mutex<Box<[FrameID]>>>>,
    width:                              usize,
    mouseState:                         &mut Flags,
    listOfParameters:                   &mut Vec<usize>,
    currentParameter:                   &mut usize,
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
            println!("=> Escape");
            *self                       =                                       TTYState::Escape;
          },
          byte @ 0x20...0x7e                                                    =>
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
            = event::Event::new
              (
                EventType::Char(byte as char),
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
          0x5b                                                                  =>
          {
            println!("=> CSI");
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
            = event::Event::new
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
              returnValue                =                                       Some(newEvent);
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
            println!("=> Parse Argument");
            *currentParameter           =                                       r as usize - 0x30;
            *self                       =                                       TTYState::ParseArgument;
          },
          0x3c                                                                  =>
          {
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
            println!("newArgument: {}", *currentParameter);
            listOfParameters.push(*currentParameter);
            *currentParameter           =                                       0;
          },
          0x4d if listOfParameters.len() == 3                                   =>
          {
            let frame: FrameID
            = if let Some(mapOfFrames) = mapOfFrames
              {
                if let Ok(mapOfFrames) = mapOfFrames.lock()
                {
                  mapOfFrames[ ( listOfParameters[1] - 1 ) * width + listOfParameters[2] - 1 ]
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
                *mouseState             |=                                      MouseButton_LeftDown;
                theEvent                =                                       Some(EventType::MouseLeftButtonPressed);
              },
              1                                                                 =>
              {
                *mouseState             |=                                      MouseButton_MiddleDown;
                theEvent                =                                       Some(EventType::MouseMiddleButtonPressed);
              },
              2                                                                 =>
              {
                *mouseState             |=                                      MouseButton_RightDown;
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
              = event::Event::new
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
          0x6d if listOfParameters.len() == 3                                  =>
          {
            let frame: FrameID
            = if let Some(mapOfFrames) = mapOfFrames
              {
                if let Ok(mapOfFrames) = mapOfFrames.lock()
                {
                  mapOfFrames[ ( listOfParameters[1] - 1 ) * width + listOfParameters[2] - 1 ]
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
                *mouseState             &=                                      !MouseButton_LeftDown;
                theEvent                =                                       Some(EventType::MouseLeftButtonReleased);
              },
              1                                                                 =>
              {
                *mouseState             &=                                      !MouseButton_MiddleDown;
                theEvent                =                                       Some(EventType::MouseMiddleButtonReleased);
              },
              2                                                                 =>
              {
                *mouseState             &=                                      !MouseButton_RightDown;
                theEvent                =                                       Some(EventType::MouseRightButtonReleased);
              },
              _                                                                 => { /* invalid */ },
            }
            *self                       =                                       TTYState::ExpectByte;
            if let Some(theEvent) = theEvent
            {
              let newEvent: Event
              = event::Event::new
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
          0x52 if listOfParameters.len() == 2                                   =>
          {
            println!("size: {}x{}", listOfParameters[2], listOfParameters[1]);
          },
          r @ _                                                                 =>
          {
           println!("cannot parse ({:?}) {}", listOfParameters, r as char);
           *self                       =                                        TTYState::ExpectByte;
          },
        }
      },
    }
    returnValue
  }
}
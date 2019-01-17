extern crate termion;
extern crate unicode_segmentation;

use std::
{
  io::
  {
    self,
    Stdin,
    Stdout,
    Write,
  },
  mem,
  rc::
  {
    Rc,
    Weak,
  },
  str::
  {
  },
  sync::
  {
    mpsc::
    {
      channel,
      Receiver,
      Sender,
    },
    RwLock,
  },
  thread,
  time::
  {
    Duration,
    SystemTime
  },
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
use unicode_segmentation::
{
  Graphemes,
  UnicodeSegmentation,
};






type YFlags                             =                                       u32;

pub enum YKeyState
{
  Pressed,
  Released,
}

pub enum YEventType
{
  Esc,

  Backspace,
  Return,

  Left,
  Right,
  Up,
  Down,

  Pause,
  Insert,
  Delete,
  Home,
  End,
  PageUp,
  PageDown,


  F(u8),
  Char(char),
  Alt(char),
  Ctrl(char),
}

pub struct YEvent
{
  event:                                YEventType,
  display:                              DisplayYUID,
  frame:                                FrameYUID,
  cursorX:                              usize,
  cursorY:                              usize,
}

type FrameYUID                          =                                       usize;

pub const Frame_None: YFlags            =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;

pub struct StatusFrame
{
  pub flags:                            YFlags,
  pub offs:                             isize,
  pub text:                             String,
  pub bgChar:                           char,
}

pub struct TextFrame
{
  pub flags:                            YFlags,
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub lines:                            Vec<String>,
  pub bgChar:                           char,
}

pub enum YColour
{
  Default,
  RGB       ( u8, u8, u8 ),             // ( r, g, b ): 0–255, 0–255, 0–255
  Standard  ( u8 ),                     // console: 0–7
  Bright    ( u8 ),                     // console: 0–7
  Cube      ( u8, u8, u8 ),             // ( r, g, b ): 0–5, 0–5, 0–5
  Grey      ( u8 ),                     // console: 0–23
  Black,                                // #000000
  Red,                                  // #ff0000
  Green,                                // #00ff00
  Brown,
  Blue,                                 // #0000ff
  Purple,                               // #ff00ff
  Cyan,                                 // #00ffff
  LightGrey,
  DarkGrey,
  LightRed,
  LightGreen,
  Yellow,                               // #ffff00
  LightBlue,
  LightPurple,
  LightCyan,
  White,                                // #ffffff
  BrightBlack,
  BrightRed,
  BrightGreen,
  BrightYellow,
  BrightBlue,
  BrightPurple,
  BrightCyan,
  BrightWhite,
  FaintBlack,
  FaintRed,
  FaintGreen,
  FaintYellow,
  FaintBlue,
  FaintPurple,
  FaintCyan,
  FaintWhite,
}

pub const YWord_None:            YFlags =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;
pub const YWord_Italic:          YFlags =                                       0b0000_0000_0000_0000_0000_0000_0000_0001;
pub const YWord_Underline:       YFlags =                                       0b0000_0000_0000_0000_0000_0000_0000_0010;
pub const YWord_SlowBlink:       YFlags =                                       0b0000_0000_0000_0000_0000_0000_0000_0100;
pub const YWord_RapidBlink:      YFlags =                                       0b0000_0000_0000_0000_0000_0000_0000_1000;
pub const YWord_Inverse:         YFlags =                                       0b0000_0000_0000_0000_0000_0000_0001_0000;
pub const YWord_Conceal:         YFlags =                                       0b0000_0000_0000_0000_0000_0000_0010_0000;
pub const YWord_CrossedOut:      YFlags =                                       0b0000_0000_0000_0000_0000_0000_0100_0000;
pub const YWord_Fraktur:         YFlags =                                       0b0000_0000_0000_0000_0000_0000_1000_0000;
pub const YWord_DoubleUnderline: YFlags =                                       0b0000_0000_0000_0000_0000_0001_0000_0000;
pub const YWord_Framed:          YFlags =                                       0b0000_0000_0000_0000_0000_0010_0000_0000;
pub const YWord_Encircled:       YFlags =                                       0b0000_0000_0000_0000_0000_0100_0000_0000;
pub const YWord_Overlined:       YFlags =                                       0b0000_0000_0000_0000_0000_1000_0000_0000;
pub struct YWord
{
  word:                                 String,
  flags:                                YFlags,
  font:                                 u8,
  fgColour:                             YColour,
  bgColour:                             YColour,
}

impl YWord
{
  pub fn new
  (
    word:                               String,
    flags:                              YFlags,
    font:                               u8,
    fgColour:                           YColour,
    bgColour:                           YColour,
  ) -> Self
  {
    Self
    {
      word:                             word,
      flags:                            flags,
      font:                             font,
      fgColour:                         fgColour,
      bgColour:                         bgColour,
    }
  }
}

pub struct EditorFrame
{
  pub flags:                            YFlags,
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub lines:                            Vec<Vec<YWord>>,
  pub bgChar:                           char,
}

pub enum BitMapData
{
  None,
  RGB   ( u8, u8, u8      ),
  RGBA  ( u8, u8, u8, u8  ),
}

pub enum BitMapEncoding
{
  None,
  Sixel(String),
}

pub struct BitMapFrame
{
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub sizeX:                            usize,
  pub sizeY:                            usize,
  pub scale:                            f64,
  pub ground:                           YColour,
  pub pixel:                            Box<[BitMapData]>,
  pub changed:                          bool,
  pub encInput:                         BitMapEncoding,
  pub encOutput:                        BitMapEncoding,
}

pub struct PlotFrame
{
}

pub enum Tiling
{
  None,
  Grid,
}

type GridBorder                         =                                       u16;

pub struct YInstance
{
  pub frame:                            FrameYUID,
  pub posX:                             isize,
  pub posY:                             isize,
  pub lenX:                             usize,
  pub lenY:                             usize,
  pub minX:                             usize,
  pub minY:                             usize,
  pub maxX:                             usize,
  pub maxY:                             usize,
  pub gridOriginX:                      GridBorder,
  pub gridOriginY:                      GridBorder,
  pub gridLenghtX:                      GridBorder,
  pub gridLenghtY:                      GridBorder,
}

pub struct ParentFrame
{
  pub typeOfTiling:                     Tiling,
  pub listOfInstances:                  Vec<YInstance>,
  pub gridBordersX:                     Vec<usize>,
  pub gridBordersY:                     Vec<usize>,
  pub pivotFrame:                       FrameYUID,
}

pub struct LayerFrame
{
  pub listOfLayers:                     Vec<FrameYUID>,
}

pub enum YFrame
{
  Status(StatusFrame),
  Text(TextFrame),
  Editor(EditorFrame),
  BitMap(BitMapFrame),
  Plot(PlotFrame),
  Parent(ParentFrame),
  Layers(LayerFrame),
}

type DisplayYUID                        =                                       usize;

pub enum ConsoleOutputType
{
  Fileio(std::fs::File),
  Stdin(std::io::Stdin),
  Stdout(std::io::Stdout),
}

pub struct ConsoleOutput
{
  input:                                ConsoleOutputType,
  output:                               ConsoleOutputType,
}

pub struct TermionOutput
{
  output:                               termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>,
  pub events:                           std::sync::mpsc::Receiver<termion::event::Event>,
  stdinThread:                          std::thread::JoinHandle<()>,
}

pub enum YDisplayType
{
  Console(ConsoleOutput),
  Termion(TermionOutput),
}

pub const YDisplay_None: YFlags         =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;
pub struct YDisplay
{
  flags:                                YFlags,
  offsX:                                isize,
  offsY:                                isize,
  sizeX:                                usize,
  sizeY:                                usize,
  cursorX:                              usize,
  cursorY:                              usize,
  mapOfFrames:                          Box<[FrameYUID]>,
  mainFrame:                            FrameYUID,
  pub needRemap:                        bool,
  pub needRefresh:                      bool,
  liveRefresh:                          bool,
  lastRefresh:                          SystemTime,
  nextRefresh:                          Duration,
  pub display:                          YDisplayType,
}

pub struct Yatui
{
  pub listOfDisplays:                   Vec<Option<YDisplay>>,
  pub listOfFrames:                     Vec<Option<YFrame>>,
  //pub recvChannel:                      std::sync::mpsc::Receiver<YEvent>,
  //pub sendChannel:                      std::sync::mpsc::Sender<YEvent>,
}

impl Yatui
{
  pub fn new
  (
  ) -> Self
  {
    Self
    {
      listOfDisplays:                   vec!(),
      listOfFrames:                     vec!(),
    }
  }
  
  pub fn newConsole
  (
    &mut self,
    offsX:                              isize,
    offsY:                              isize,
    input:                              ConsoleOutputType,
    output:                             ConsoleOutputType,
    refreshInRealTime:                  bool,
    refreshRate:                        u64,
  ) -> ( usize, usize, DisplayYUID )
  {
    ( 0, 0, 0 )
  }
  pub fn newTermion
  (
    &mut self,
    offsX:                              isize,
    offsY:                              isize,
    input:                              Stdin,
    output:                             Stdout,
    refreshInRealTime:                  bool,
    refreshRate:                        u64,
  ) -> ( usize, usize, DisplayYUID )
  {
    let ( sizeX, sizeY ): ( u16, u16 )  =                                       termion::terminal_size().unwrap();
    let ( txEvents, rxEvents ): ( Sender<Event>, Receiver<Event> )
                                        =                                       channel();
    let output
    = TermionOutput
      {
        output:                         MouseTerminal::from(output.into_raw_mode().unwrap()),
        events:                         rxEvents,
        stdinThread:
        thread::spawn
        (
          move ||
          {
            let stdin                   =                                       input;
            for event                   in                                      stdin.events()
            {
              if event.is_ok()
              {
                txEvents.send(event.unwrap()).unwrap();
              }
            }
          }
        ),
      };
    let mut mapOfFrames: Vec<FrameYUID> =                                       Vec::with_capacity( ( sizeX * sizeY ) as usize );
    mapOfFrames.resize(( sizeX * sizeY ) as usize, 0);
    let mapOfFrames: Box<[FrameYUID]>   =                                       mapOfFrames.into_boxed_slice();
    self.listOfDisplays.push
    (
      Some
      (
        YDisplay
        {
          flags:                        YDisplay_None,
          offsX:                        offsX,
          offsY:                        offsY,
          sizeX:                        sizeX as usize,
          sizeY:                        sizeY as usize,
          cursorX:                      1,
          cursorY:                      1,
          mapOfFrames:                  mapOfFrames,
          mainFrame:                    0,
          needRemap:                    true,
          needRefresh:                  true,
          liveRefresh:                  refreshInRealTime,
          lastRefresh:                  SystemTime::now(),
          nextRefresh:                  Duration::from_nanos(refreshRate),
          display:                      YDisplayType::Termion ( output ),
        }
      )
    );
    (
      sizeX as usize,
      sizeY as usize,
      self.listOfDisplays.len(),
    )
  }
  
  pub fn newStatusFrame
  (
    &mut self,
    flags:                              YFlags,
    offs:                               isize,
    text:                               String,
    bgChar:                             char,
  ) -> FrameYUID
  {
    self.listOfFrames.push
    (
      Some
      (
        YFrame::Status
        (
          StatusFrame
          {
            flags:                      flags,
            offs:                       offs,
            text:                       text,
            bgChar:                     bgChar,
          }
        )
      )
    );
    self.listOfFrames.len()
  }
  
  pub fn swapFrame
  (
    listOfFrames:                       &mut Vec<Option<YFrame>>,
    uidFrame:                           FrameYUID,
    mut newFrame:                       Option<YFrame>,
  ) -> Result<Option<YFrame>, &str>
  {
    if uidFrame == 0
    {
      Err("UID of frame cannot be zero")
    }
    else if uidFrame > listOfFrames.len()
    {
      Err("UID of frame too high")
    }
    else
    {
      mem::swap
      (
        &mut listOfFrames[ uidFrame - 1 ],
        &mut newFrame
      );
      Ok(newFrame)
    }
  }

  pub fn newTextFrame
  (
    &mut self,
    flags:                              YFlags,
    offsX:                              isize,
    offsY:                              isize,
    lines:                              Vec<String>,
    bgChar:                             char,
  ) -> FrameYUID
  {
    self.listOfFrames.push
    (
      Some
      (
        YFrame::Text
        (
          TextFrame
          {
            flags:                      flags,
            offsX:                      offsX,
            offsY:                      offsY,
            lines:                      lines,
            bgChar:                     bgChar,
          }
        )
      )
    );
    self.listOfFrames.len()
  }

  pub fn newEditorFrame
  (
    &mut self,
    flags:                              YFlags,
    offsX:                              isize,
    offsY:                              isize,
    lines:                              Vec<Vec<YWord>>,
    bgChar:                             char,
  ) -> FrameYUID
  {
    self.listOfFrames.push
    (
      Some
      (
        YFrame::Editor
        (
          EditorFrame
          {
            flags:                      flags,
            offsX:                      offsX,
            offsY:                      offsY,
            lines:                      lines,
            bgChar:                     bgChar,
          }
        )
      )
    );
    self.listOfFrames.len()
  }

  pub fn newBitMapFrame
  (
    &mut self,
    offsX:                              isize,
    offsY:                              isize,
    sizeX:                              usize,
    sizeY:                              usize,
    scale:                              f64,
    ground:                             YColour,
    pixel:                              Box<[BitMapData]>,
    encInput:                           BitMapEncoding,
    encOutput:                          BitMapEncoding,
  ) -> FrameYUID
  {
    self.listOfFrames.push
    (
      Some
      (
        YFrame::BitMap
        (
          BitMapFrame
          {
            offsX:                      offsX,
            offsY:                      offsY,
            sizeX:                      sizeX,
            sizeY:                      sizeY,
            scale:                      scale,
            ground:                     ground,
            pixel:                      pixel,
            changed:                    true,
            encInput:                   encInput,
            encOutput:                  encOutput,
          }
        )
      )
    );
    self.listOfFrames.len()
  }

  pub fn newParentFrame
  (
    &mut self,
    tiling:                             Tiling,
    listOfInstances:                    Vec<YInstance>,
    gridBordersX:                       Vec<usize>,
    gridBordersY:                       Vec<usize>,
    pivotFrame:                         FrameYUID,
  ) -> FrameYUID
  {
    self.listOfFrames.push
    (
      Some
      (
        YFrame::Parent
        (
          ParentFrame
          {
            typeOfTiling:               tiling,
            listOfInstances:            listOfInstances,
            gridBordersX:               gridBordersX,
            gridBordersY:               gridBordersY,
            pivotFrame:                 pivotFrame,
          }
        )
      )
    );
    self.listOfFrames.len()
  }

  pub fn newInstance
  (
    frame:                              FrameYUID,
    posX:                               isize,
    posY:                               isize,
    lenX:                               usize,
    lenY:                               usize,
    minX:                               usize,
    minY:                               usize,
    maxX:                               usize,
    maxY:                               usize,
    gridOriginX:                        GridBorder,
    gridOriginY:                        GridBorder,
    gridLenghtX:                        GridBorder,
    gridLenghtY:                        GridBorder,
  ) -> YInstance
  {
    YInstance
    {
      frame:                            frame,
      posX:                             posX,
      posY:                             posY,
      lenX:                             lenX,
      lenY:                             lenY,
      minX:                             minX,
      minY:                             minY,
      maxX:                             maxX,
      maxY:                             maxY,
      gridOriginX:                      gridOriginX,
      gridOriginY:                      gridOriginY,
      gridLenghtX:                      gridLenghtX,
      gridLenghtY:                      gridLenghtY,
    }
  }
  
  fn accessFrame
  (
    listOfFrames:                       &mut Vec<Option<YFrame>>,
    frame:                              FrameYUID,
  ) -> Result<&mut YFrame, &str>
  {
    if frame == 0
    {
      Err("UID of frame cannot be zero")
    }
    else if frame > listOfFrames.len()
    {
      Err("UID of frame too high")
    }
    else if let Some(ref mut f) = listOfFrames [ frame - 1 ]
    {
      Ok(f)
    }
    else
    {
      Err("UID of frame invalidated.")
    }
  }

  fn enquireFrame
  (
    listOfFrames:                       &Vec<Option<YFrame>>,
    frame:                              FrameYUID,
  ) -> Result<&YFrame, &str>
  {
    if frame == 0
    {
      Err("UID of frame cannot be zero")
    }
    else if frame > listOfFrames.len()
    {
      Err("UID of frame too high")
    }
    else if let Some(ref f) = listOfFrames [ frame - 1 ]
    {
      Ok(f)
    }
    else
    {
      Err("UID of frame invalidated.")
    }
  }

  fn accessDisplay
  (
    listOfDisplays:                     &mut Vec<Option<YDisplay>>,
    display:                            DisplayYUID,
  ) -> Result<&mut YDisplay, &str>
  {
    if display == 0
    {
      Err("UID of display cannot be zero")
    }
    else if display > listOfDisplays.len()
    {
      Err("UID of display too high")
    }
    else if let Some(ref mut d) = listOfDisplays [ display - 1 ]
    {
      Ok(d)
    }
    else
    {
      Err("UID of display invalidated.")
    }
  }

  fn enquireDisplay
  (
    listOfDisplays:                     &Vec<Option<YDisplay>>,
    display:                            DisplayYUID,
  ) -> Result<&YDisplay, &str>
  {
    if display == 0
    {
      Err("UID of display cannot be zero")
    }
    else if display > listOfDisplays.len()
    {
      Err("UID of display too high")
    }
    else if let Some(ref d) = listOfDisplays [ display - 1 ]
    {
      Ok(d)
    }
    else
    {
      Err("UID of display invalidated.")
    }
  }

  fn draw
  (
    &mut self,
    display:                            &mut YDisplay,
    //lstOfFrames:                        &Vec<Option<YFrame>>,
    renderFrame:                        FrameYUID,
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
    if ( minX <= display.sizeX as isize ) && ( minY <= display.sizeY as isize )
    {
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
      if maxX > display.sizeX as isize
      {
        maxX                            =                                       display.sizeX as isize;
      }
      if maxY > display.sizeY as isize
      {
        maxY                            =                                       display.sizeY as isize;
      }
      let minX: usize                   =                                       minX as usize;
      let minY: usize                   =                                       minY as usize;
      let maxX: usize                   =                                       maxX as usize;
      let maxY: usize                   =                                       maxY as usize;
      let lenX: usize                   =                                       ( maxX - minX )  as usize;
      let lenY: usize                   =                                       ( maxY - minY )  as usize;
      let refFrame                      =                                       Yatui::enquireFrame ( &mut self.listOfFrames, renderFrame ).unwrap();
      if display.needRemap
      {
        display.needRemap               =                                       false;
        for y                           in                                      minY .. maxY
        {
          for x                         in                                      minX .. maxX
          {
            display.mapOfFrames [ x + y * display.sizeX ]
                                        =                                       renderFrame;
          }
        }
      }
      match refFrame
      {
        YFrame::Status ( ref frame ) =>
        {
          match display.display
          {
            YDisplayType::Console(ref mut output) =>
            {
            },
            YDisplayType::Termion(ref mut output) =>
            {
              let empty                 =                                       frame.bgChar.to_string().repeat( lenX );
              write!
              (
                output.output,
                "{}{}",
                termion::cursor::Goto
                (
                  ( minX + 1 ) as u16,
                  ( minY + 1 ) as u16,
                ),
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
                  "{}",
                  termion::cursor::Goto
                  (
                    ( minX + 1  + shift ) as u16, 
                    ( minY + 1          ) as u16,
                  ),
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
        YFrame::Text ( ref frame ) =>
        {
          match display.display
          {
            YDisplayType::Console(ref mut output) =>
            {
            },
            YDisplayType::Termion(ref mut output) =>
            {
              let empty                 =                                       frame.bgChar.to_string().repeat( lenX );
              {
                let posX: u16           =                                       ( minX + 1 ) as u16;
                for posY                in                                      ( minY + 1 ) as u16 .. ( maxY + 1 ) as u16
                {
                  write!
                  (
                    output.output,
                    "{}{}",
                    termion::cursor::Goto
                    (
                      posX,             posY,
                    ),
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
                      "{}",
                      termion::cursor::Goto
                      (
                        ( posX         ) as u16, 
                        ( posY + index ) as u16,
                      ),
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
        YFrame::Editor ( ref frame ) =>
        {
          match display.display
          {
            YDisplayType::Console(ref mut output) =>
            {
            },
            YDisplayType::Termion(ref mut output) =>
            {
              let empty                 =                                       frame.bgChar.to_string().repeat( lenX );
              {
                let posX: u16           =                                       ( minX + 1 ) as u16;
                for posY                in                                      ( minY + 1 ) as u16 .. ( maxY + 1 ) as u16
                {
                  write!
                  (
                    output.output,
                    "{}{}",
                    termion::cursor::Goto
                    (
                      posX,             posY,
                    ),
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
                      "{}",
                      termion::cursor::Goto
                      (
                        ( posX         ) as u16, 
                        ( posY + index ) as u16,
                      ),
                    ).unwrap();
                    for word            in                                      words
                    {
                      let mut style     =                                       "".to_string();
                      if ( word.font > 0 ) && ( word.font < 10 )                { style = format!("{};", word.font + 10 ) }
                      if ( word.flags & YWord_Italic          ) == 1            { style.push_str("3;") }
                      if ( word.flags & YWord_Underline       ) == 1            { style.push_str("4;") }
                      if ( word.flags & YWord_SlowBlink       ) == 1            { style.push_str("5;") }
                      if ( word.flags & YWord_RapidBlink      ) == 1            { style.push_str("6;") }
                      if ( word.flags & YWord_Inverse         ) == 1            { style.push_str("7;") }
                      if ( word.flags & YWord_Conceal         ) == 1            { style.push_str("8;") }
                      if ( word.flags & YWord_CrossedOut      ) == 1            { style.push_str("9;") }
                      if ( word.flags & YWord_Fraktur         ) == 1            { style.push_str("20;") }
                      if ( word.flags & YWord_DoubleUnderline ) == 1            { style.push_str("21;") }
                      if ( word.flags & YWord_Framed          ) == 1            { style.push_str("51;") }
                      if ( word.flags & YWord_Encircled       ) == 1            { style.push_str("52;") }
                      if ( word.flags & YWord_Overlined       ) == 1            { style.push_str("53;") }
                      let fgColour
                      = match word.fgColour
                        {
                          YColour::Default                  => { "39".to_string() },
                          YColour::RGB( red, green, blue )  => { format!( "38;2;{};{};{}", red, green, blue) },
                          YColour::Standard( colour )       if colour < 8
                                                            => { format!( "38;5;{}", colour ) },
                          YColour::Bright( colour )         if colour < 8
                                                            => { format!( "38;5;{}", colour + 8 ) },
                          YColour::Cube( red, green, blue ) if red < 6
                                                            && green < 6
                                                            && blue < 6
                                                            => { format!( "38;5;{}", 16 + 36 * red + 6 * green + blue) },
                          YColour::Grey( colour )           if colour < 24
                                                            => { format!( "38;5;{}", colour + 232 ) },
                          YColour::Black                    => { "30".to_string() },
                          YColour::Red                      => { "31".to_string() },
                          YColour::Green                    => { "32".to_string() },
                          YColour::Brown                    => { "33".to_string() },
                          YColour::Blue                     => { "34".to_string() },
                          YColour::Purple                   => { "35".to_string() },
                          YColour::Cyan                     => { "36".to_string() },
                          YColour::LightGrey                => { "37".to_string() },
                          YColour::DarkGrey                 => { "1;30".to_string() },
                          YColour::LightRed                 => { "1;31".to_string() },
                          YColour::LightGreen               => { "1;32".to_string() },
                          YColour::Yellow                   => { "1;33".to_string() },
                          YColour::LightBlue                => { "1;34".to_string() },
                          YColour::LightPurple              => { "1;35".to_string() },
                          YColour::LightCyan                => { "1;36".to_string() },
                          YColour::White                    => { "1;37".to_string() },
                          YColour::FaintBlack               => { "2;30".to_string() },
                          YColour::FaintRed                 => { "2;31".to_string() },
                          YColour::FaintGreen               => { "2;32".to_string() },
                          YColour::FaintYellow              => { "2;33".to_string() },
                          YColour::FaintBlue                => { "2;34".to_string() },
                          YColour::FaintPurple              => { "2;35".to_string() },
                          YColour::FaintCyan                => { "2;36".to_string() },
                          YColour::FaintWhite               => { "2;37".to_string() },
                          YColour::BrightBlack              => { "90".to_string() },
                          YColour::BrightRed                => { "91".to_string() },
                          YColour::BrightGreen              => { "92".to_string() },
                          YColour::BrightYellow             => { "93".to_string() },
                          YColour::BrightBlue               => { "94".to_string() },
                          YColour::BrightPurple             => { "95".to_string() },
                          YColour::BrightCyan               => { "96".to_string() },
                          YColour::BrightWhite              => { "97".to_string() },
                          _                                 => { "".to_string()  },
                        };
                      let bgColour
                      = match word.bgColour
                        {
                          YColour::Default                  => { "49".to_string() },
                          YColour::RGB( red, green, blue )  => { format!( "48;2;{};{};{}", red, green, blue ) },
                          YColour::Standard( colour )       if colour < 8
                                                            => { format!( "48;5;{}", colour ) },
                          YColour::Bright( colour )         if colour < 8
                                                            => { format!( "48;5;{}", colour + 8 ) },
                          YColour::Cube( red, green, blue ) if red < 6
                                                            && green < 6
                                                            && blue < 6
                                                            => { format!( "48;5;{}", 16 + 36 * red + 6 * green + blue) },
                          YColour::Grey( colour )           if colour < 24
                                                            => { format!( "48;5;{}", colour + 232 ) },
                          YColour::Black                    => { "40".to_string() },
                          YColour::Red                      => { "41".to_string() },
                          YColour::Green                    => { "42".to_string() },
                          YColour::Brown                    => { "43".to_string() },
                          YColour::Blue                     => { "44".to_string() },
                          YColour::Purple                   => { "45".to_string() },
                          YColour::Cyan                     => { "46".to_string() },
                          YColour::LightGrey                => { "47".to_string() },
                          YColour::DarkGrey                 => { "1;40".to_string() },
                          YColour::LightRed                 => { "1;41".to_string() },
                          YColour::LightGreen               => { "1;42".to_string() },
                          YColour::Yellow                   => { "1;43".to_string() },
                          YColour::LightBlue                => { "1;44".to_string() },
                          YColour::LightPurple              => { "1;45".to_string() },
                          YColour::LightCyan                => { "1;46".to_string() },
                          YColour::White                    => { "1;47".to_string() },
                          YColour::FaintBlack               => { "2;40".to_string() },
                          YColour::FaintRed                 => { "2;41".to_string() },
                          YColour::FaintGreen               => { "2;42".to_string() },
                          YColour::FaintYellow              => { "2;43".to_string() },
                          YColour::FaintBlue                => { "2;44".to_string() },
                          YColour::FaintPurple              => { "2;45".to_string() },
                          YColour::FaintCyan                => { "2;46".to_string() },
                          YColour::FaintWhite               => { "2;47".to_string() },
                          YColour::BrightBlack              => { "100".to_string() },
                          YColour::BrightRed                => { "101".to_string() },
                          YColour::BrightGreen              => { "102".to_string() },
                          YColour::BrightYellow             => { "103".to_string() },
                          YColour::BrightBlue               => { "104".to_string() },
                          YColour::BrightPurple             => { "105".to_string() },
                          YColour::BrightCyan               => { "106".to_string() },
                          YColour::BrightWhite              => { "107".to_string() },
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
        YFrame::BitMap(ref frame) =>
        {
          //TODO
          match display.display
          {
            YDisplayType::Console(ref mut output) =>
            {
            },
            YDisplayType::Termion(ref mut output) =>
            {
            },
          }
        },
        YFrame::Plot( ref frame ) =>
        {
          //TODO
          match display.display
          {
            YDisplayType::Console(ref mut output) =>
            {
            },
            YDisplayType::Termion(ref mut output) =>
            {
            },
          }
        },
        YFrame::Parent ( ref frame ) =>
        {
          for instance                  in                                      &frame.listOfInstances
          {
            let posX                    =                                       posX + instance.posX;
            let posY                    =                                       posY + instance.posY;
            let lenX                    =                                       instance.lenX;
            let lenY                    =                                       instance.lenY;
            let next                    =                                       instance.frame;
            self.draw
            (
              display,
              //lstOfFrames,
              next,
              posX,
              posY,
              lenX,
              lenY,
            );
          }
        },
        YFrame::Layers( ref frame ) =>
        {
          //TODO
        },
      }
    }
  }

  pub fn render
  (
    &mut self,
  )
  {
    //let listOfFrame                     =                                       &mut self.listOfFrames;
    for display                         in                                      &mut self.listOfDisplays
    {
      if let Some(ref mut refDisplay) = display
      {
        if  ( refDisplay.needRefresh || refDisplay.liveRefresh )                &&
            ( refDisplay.lastRefresh.elapsed().unwrap() > refDisplay.nextRefresh )
        {
          refDisplay.needRefresh        =                                       false;
          self.draw
          (
            refDisplay,
            //listOfFrame,
            refDisplay.mainFrame,
            refDisplay.offsX,           refDisplay.offsY,
            refDisplay.sizeX,           refDisplay.sizeY,
          );
          match refDisplay.display
          {
            YDisplayType::Console(ref mut output) =>
            {
            },
            YDisplayType::Termion(ref mut output) =>
            {
              output.output.flush().unwrap();
              let ( sizeX, sizeY )      =                                       termion::terminal_size().unwrap();
              if ( sizeX as usize != refDisplay.sizeX ) || ( sizeY as usize != refDisplay.sizeY )
              {
                //resized
              }
            }
          }
          refDisplay.lastRefresh        =                                       SystemTime::now();
        }
      }
    }
  }

  pub fn turnOffDisplay
  (
    &mut self,
    display:                            DisplayYUID,
  )
  {
    let refDisplay                      =                                       Yatui::accessDisplay ( &mut self.listOfDisplays, display ).unwrap();
    match refDisplay.display
    {
      YDisplayType::Console(ref mut output) =>
      {
      },
      YDisplayType::Termion(ref mut output) =>
      {
        write!
        (
          output.output,
          "\x1bc",
        ).unwrap();
        output.output.flush().unwrap();
      }
    }
  }

  pub fn turnOnDisplay
  (
    &mut self,
    display:                            DisplayYUID,
    frame:                              FrameYUID,
  )
  {
    let     refFrame                    =                                       Yatui::accessFrame    ( &mut self.listOfFrames,   frame   ).unwrap();
    let mut refDisplay                  =                                       Yatui::accessDisplay  ( &mut self.listOfDisplays, display ).unwrap();
    refDisplay.mainFrame                =                                       frame;
    match refDisplay.display
    {
      YDisplayType::Console(ref mut output) =>
      {
      },
      YDisplayType::Termion(ref mut output) =>
      {
        write!
        (
          output.output,
          "\x1b]0;{}\x07{}{}{}xyz",
          "Hello World",
          termion::clear::All,
          termion::cursor::Goto(1, 1),
          termion::cursor::Hide,
        ).unwrap();
        output.output.flush().unwrap();
      }
    }
  }
}

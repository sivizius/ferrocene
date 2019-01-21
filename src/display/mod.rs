#[cfg(feature = "display-tty")]
pub mod tty;

use crate::
{
  Flags,
  event::
  {
    EventSender,
  },
  frame::
  {
    Frame,
    FrameID,
    Tiling,
  },
};

#[cfg(feature = "display-tty")]
use crate::display::tty::
{
  TTYDisplay,
};

use std::
{
  io::
  {
    Read,
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
      },
    },
  },
  sync::
  {
    Arc,
    Mutex,
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

pub enum DisplayType
{
  #[cfg(feature = "display-tty")]
  TTY(TTYDisplay),
}

bitflags!
{
  pub struct DisplayFlag: Flags
  {
    const None                          =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;
    const RealTime                      =                                       0b0000_0000_0000_0000_0000_0000_0000_0001;
    //…
    const NeedRefresh                   =                                       0b0100_0000_0000_0000_0000_0000_0000_0000;
    const NeedRemap                     =                                       0b1000_0000_0000_0000_0000_0000_0000_0000;
    const MaskRefresh                   =                                       DisplayFlag::NeedRefresh.bits | DisplayFlag::NeedRemap.bits;
  }
}

pub struct Display
{
  pub flags:                            DisplayFlag,
  pub this:                             DisplayID,
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub sizeX:                            usize,
  pub sizeY:                            usize,
  pub cursorX:                          usize,
  pub cursorY:                          usize,
  mapOfFrames:                          Arc<Mutex<Option<Box<[FrameID]>>>>,
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
    let mut minX:                 isize =                                       posX;
    let mut minY:                 isize =                                       posY;
    let mut maxX:                 isize =                                       posX + lenX as isize;
    let mut maxY:                 isize =                                       posY + lenY as isize;
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
      let mut cutX:               usize =                                       0;
      let mut cutY:               usize =                                       0;
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
      let minX:                   usize =                                       minX as usize;
      let minY:                   usize =                                       minY as usize;
      let maxX:                   usize =                                       maxX as usize;
      let maxY:                   usize =                                       maxY as usize;
      let lenX:                   usize =                                       ( maxX - minX )  as usize;
      let lenY:                   usize =                                       ( maxY - minY )  as usize;
      if ( self.flags & DisplayFlag::NeedRemap ) != DisplayFlag::None           //do I have to remap the map of frames?
      {
        if let Ok(mut mapOfFrames) = self.mapOfFrames.lock()                    //can I access it?
        {
          //allocate new map of frames
          let mut theMapOfFrames: Vec<FrameID>
                                        =                                       Vec::with_capacity( ( self.sizeX * self.sizeY ) as usize );
          theMapOfFrames.resize(( self.sizeX * self.sizeY ) as usize, 0);
          let mut theMapOfFrames: Box<[FrameID]>
                                        =                                       theMapOfFrames.into_boxed_slice();

          //remap
          for y                         in                                      minY .. maxY
          {
            for x                       in                                      minX .. maxX
            {
              theMapOfFrames [ x + y * self.sizeX ]
                                        =                                       drawFrame;
            }
          }

          //do not have to remap, because I did
          self.flags                    &=                                      !DisplayFlag::NeedRemap;

          //replace old map of frames
          *mapOfFrames                  =                                       Some(theMapOfFrames);
        }
      }
      match refFrame.as_mut().unwrap()
      {
        Frame::Status ( ref frame )     =>
        {
          match self.display
          {
            #[cfg(feature = "display-tty")]
            DisplayType::TTY(ref mut output)  => output.drawStatusFrame ( frame, events, lenX, lenY, minX, minY, maxY, maxY, cutX, cutY ),
          }
        },
        Frame::Text ( ref frame )       =>
        {
          match self.display
          {
            #[cfg(feature = "display-tty")]
            DisplayType::TTY(ref mut output)  => output.drawTextFrame   ( frame, events, lenX, lenY, minX, minY, maxY, maxY, cutX, cutY ),
          }
        },
        Frame::Editor ( ref frame )     =>
        {
          match self.display
          {
            #[cfg(feature = "display-tty")]
            DisplayType::TTY(ref mut output)  => output.drawEditorFrame ( frame, events, lenX, lenY, minX, minY, maxY, maxY, cutX, cutY ),
          }
        },
        Frame::Pixel( ref frame )       =>
        {
          match self.display
          {
            #[cfg(feature = "display-tty")]
            DisplayType::TTY(ref mut output)  => output.drawPixelFrame  ( frame, events, lenX, lenY, minX, minY, maxY, maxY, cutX, cutY ),
          }
        },
        Frame::Plot( ref frame )        =>
        {
          match self.display
          {
            #[cfg(feature = "display-tty")]
            DisplayType::TTY(ref mut output)  => output.drawPlotFrame   ( frame, events, lenX, lenY, minX, minY, maxY, maxY, cutX, cutY ),
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

  pub fn changeTitle
  (
    &mut self,
    events:                             EventSender,
    title:                              String,
  )
  {
    match self.display
    {
      #[cfg(feature = "display-tty")]
      DisplayType::TTY(ref mut output)  => output.changeTitle ( events, self.this, title ),
    }
  }

  pub fn turnOn
  (
    &mut self,
    events:                             EventSender,
    title:                              String,
  )
  {
    match self.display
    {
      #[cfg(feature = "display-tty")]
      DisplayType::TTY(ref mut output)  => output.turnOn      ( events, self.this, title, self.sizeX, self.sizeY, self.mapOfFrames.clone(), self.focusedFrame.clone() ),
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
      #[cfg(feature = "display-tty")]
      DisplayType::TTY(ref mut output)  => output.turnOff     ( &events, self.this ),
    }
  }
}

use crate::*;

pub type FrameID                        =                                       usize;
type GridBorder                         =                                       usize;

pub const Frame_None:             Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;
pub const Style_None:             Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;
pub const Style_Italic:           Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0001;
pub const Style_Underline:        Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0010;
pub const Style_SlowBlink:        Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0100;
pub const Style_RapidBlink:       Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_1000;
pub const Style_Inverse:          Flags =                                       0b0000_0000_0000_0000_0000_0000_0001_0000;
pub const Style_Conceal:          Flags =                                       0b0000_0000_0000_0000_0000_0000_0010_0000;
pub const Style_CrossedOut:       Flags =                                       0b0000_0000_0000_0000_0000_0000_0100_0000;
pub const Style_Fraktur:          Flags =                                       0b0000_0000_0000_0000_0000_0000_1000_0000;
pub const Style_DoubleUnderline:  Flags =                                       0b0000_0000_0000_0000_0000_0001_0000_0000;
pub const Style_Framed:           Flags =                                       0b0000_0000_0000_0000_0000_0010_0000_0000;
pub const Style_Encircled:        Flags =                                       0b0000_0000_0000_0000_0000_0100_0000_0000;
pub const Style_Overlined:        Flags =                                       0b0000_0000_0000_0000_0000_1000_0000_0000;

pub struct StatusFrame
{
  pub flags:                            Flags,
  pub offs:                             isize,
  pub text:                             String,
  pub bgChar:                           char,
}

pub struct TextFrame
{
  pub flags:                            Flags,
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub lines:                            Vec<String>,
  pub bgChar:                           char,
}

pub enum Colour
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

pub struct StyledToken
{
  pub word:                             String,
  pub flags:                            Flags,
  pub font:                             u8,
  pub fgColour:                         Colour,
  pub bgColour:                         Colour,
}

impl StyledToken
{
  pub fn new
  (
    word:                               String,
    flags:                              Flags,
    font:                               u8,
    fgColour:                           Colour,
    bgColour:                           Colour,
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
  pub flags:                            Flags,
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub lines:                            Vec<Vec<StyledToken>>,
  pub bgChar:                           char,
}

pub enum PixelEncoding
{
  None,
  RGB   ( u8, u8, u8      ),
  RGBA  ( u8, u8, u8, u8  ),
  Sixel ( String          ),
}

pub struct PixelFrame
{
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub sizeX:                            usize,
  pub sizeY:                            usize,
  pub scale:                            f64,
  pub ground:                           Colour,
  pub changed:                          bool,
  pub input:                            PixelEncoding,
  pub output:                           PixelEncoding,
}

pub struct PlotFrame
{
}

pub enum Tiling
{
  None,
  Grid,
}

pub struct Instance
{
  pub frame:                            FrameID,
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
  pub listOfInstances:                  Vec<Instance>,
  pub gridBordersX:                     Vec<isize>,
  pub gridBordersY:                     Vec<isize>,
  pub gridMinimumX:                     Vec<usize>,
  pub gridMinimumY:                     Vec<usize>,
  pub pivotFrame:                       FrameID,
}

pub struct LayerFrame
{
  pub listOfLayers:                     Vec<FrameID>,
}

pub enum Frame
{
  Status(StatusFrame),
  Text(TextFrame),
  Editor(EditorFrame),
  Pixel(PixelFrame),
  Plot(PlotFrame),
  Parent(ParentFrame),
  Layers(LayerFrame),
}

impl Frame
{
  pub fn newStatusFrame
  (
    flags:                              Flags,
    offs:                               isize,
    text:                               String,
    bgChar:                             char,
  ) -> Frame
  {
    Frame::Status
    (
      StatusFrame
      {
        flags:                          flags,
        offs:                           offs,
        text:                           text,
        bgChar:                         bgChar,
      }
    )
  }

  pub fn newTextFrame
  (
    flags:                              Flags,
    offsX:                              isize,
    offsY:                              isize,
    lines:                              Vec<String>,
    bgChar:                             char,
  ) -> Frame
  {
    Frame::Text
    (
      TextFrame
      {
        flags:                          flags,
        offsX:                          offsX,
        offsY:                          offsY,
        lines:                          lines,
        bgChar:                         bgChar,
      }
    )
  }

  pub fn newEditorFrame
  (
    flags:                              Flags,
    offsX:                              isize,
    offsY:                              isize,
    lines:                              Vec<Vec<StyledToken>>,
    bgChar:                             char,
  ) -> Frame
  {
    Frame::Editor
    (
      EditorFrame
      {
        flags:                          flags,
        offsX:                          offsX,
        offsY:                          offsY,
        lines:                          lines,
        bgChar:                         bgChar,
      }
    )
  }

  pub fn newPixelFrame
  (
    offsX:                              isize,
    offsY:                              isize,
    sizeX:                              usize,
    sizeY:                              usize,
    scale:                              f64,
    ground:                             Colour,
    input:                              PixelEncoding,
    output:                             PixelEncoding,
  ) -> Frame
  {
    Frame::Pixel
    (
      PixelFrame
      {
        offsX:                          offsX,
        offsY:                          offsY,
        sizeX:                          sizeX,
        sizeY:                          sizeY,
        scale:                          scale,
        ground:                         ground,
        changed:                        true,
        input:                          input,
        output:                         output,
      }
    )
  }

  pub fn newInstance
  (
    frame:                              FrameID,
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
  ) -> Instance
  {
    Instance
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

  pub fn newParentFrame
  (
    tiling:                             Tiling,
    listOfInstances:                    Vec<Instance>,
    mut gridBordersX:                   Vec<isize>,
    mut gridBordersY:                   Vec<isize>,
    mut gridMinimumX:                   Vec<usize>,
    mut gridMinimumY:                   Vec<usize>,
    pivotFrame:                         FrameID,
  ) -> Frame
  {
    gridBordersX.sort();
    gridBordersY.sort();
    gridMinimumX.resize( gridBordersX.len() - 1, 0 );
    gridMinimumY.resize( gridBordersY.len() - 1, 0 );
    Frame::Parent
    (
      ParentFrame
      {
        typeOfTiling:                   tiling,
        listOfInstances:                listOfInstances,
        gridBordersX:                   gridBordersX,
        gridBordersY:                   gridBordersY,
        gridMinimumX:                   gridMinimumX,
        gridMinimumY:                   gridMinimumY,
        pivotFrame:                     pivotFrame,
      }
    )
  }
}
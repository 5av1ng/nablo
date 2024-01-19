//! storges events that we care

use crate::texture::Image;
use nablo_shape::math::Vec2;
#[cfg(feature = "manager")]
use winit::event::ElementState;
#[cfg(feature = "manager")]
use winit::keyboard::KeyCode;
#[cfg(feature = "manager")]
use winit::event::WindowEvent;
#[cfg(feature = "manager")]
use winit::keyboard::PhysicalKey;

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
/// events the host should handle
pub enum OutputEvent {
	TextureCreate(Image),
	TextureChange(Image),
	TextureDelete(String),
	ClipboardCopy(String),
	/// true for open
	RequireSoftKeyboard(bool),
}

/// storges events that we care
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Event {
	/// contains which has been pressed and what charater it produced
	KeyPressed(Key),
	/// contains which has been released
	KeyRelease(Key),
	/// contains where the cursor is
	CursorMoved(Vec2),
	CursorEntered,
	CursorLeft,
	/// contains which button have been clicked
	MouseClick(MouseButton),
	/// contains which button have been relased
	MouseRelease(MouseButton),
	Resized(Vec2),
	ImeEnable,
	ImeDisable,
	TextInput(String),
	TouchStart(Touch),
	TouchMove(Touch),
	TouchEnd(Touch),
	TouchCancel(Touch),
	/// contains scroll delta
	Scroll(Vec2),
	#[default] NotSupported,
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Default)]
/// a stuct for touch
pub struct Touch {
	/// touch id
	pub id: usize,
	pub location: Vec2,
	pub phase: TouchPhase
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum TouchPhase {
	Start,
	Move,
	#[default] End,
}

/// stands for mouse Buttuons
#[derive(Clone, Debug, PartialEq, Default, Eq, Hash)]
pub enum MouseButton {
	#[default] Left,
	Right,
	Middle,
	Back,
	Forward,
	Other(usize),
}

#[cfg(feature = "manager")]
impl Into<Event> for WindowEvent {
	fn into(self) -> Event { 
		match self {
			Self::KeyboardInput{event, is_synthetic, .. } => {
				if !is_synthetic {
					match event.state {
						winit::event::ElementState::Pressed => Event::KeyPressed(event.physical_key.into()),
						winit::event::ElementState::Released => Event::KeyRelease(event.physical_key.into()),
					}
				}else {
					Event::NotSupported
				}
			},
			Self::CursorMoved{ position, .. } => Event::CursorMoved(Vec2::new(position.x as f32, position.y as f32)),
			Self::CursorEntered{..} => Event::CursorEntered,
			Self::CursorLeft{..} => Event::CursorLeft,
			Self::MouseInput{state, button, ..} => {
				match state {
					ElementState::Pressed => {
						match button {
							winit::event::MouseButton::Left => Event::MouseClick(MouseButton::Left),
							winit::event::MouseButton::Right => Event::MouseClick(MouseButton::Right),
							winit::event::MouseButton::Middle => Event::MouseClick(MouseButton::Middle),
							winit::event::MouseButton::Back => Event::MouseClick(MouseButton::Back),
							winit::event::MouseButton::Forward => Event::MouseClick(MouseButton::Forward),
							winit::event::MouseButton::Other(t) => Event::MouseClick(MouseButton::Other(t.into())),
						}
					},
					ElementState::Released => {
						match button {	
							winit::event::MouseButton::Left => Event::MouseRelease(MouseButton::Left),
							winit::event::MouseButton::Right => Event::MouseRelease(MouseButton::Right),
							winit::event::MouseButton::Middle => Event::MouseRelease(MouseButton::Middle),
							winit::event::MouseButton::Back => Event::MouseRelease(MouseButton::Back),
							winit::event::MouseButton::Forward => Event::MouseRelease(MouseButton::Forward),
							winit::event::MouseButton::Other(t) => Event::MouseRelease(MouseButton::Other(t.into())),
				    	}
					},
				}
			},
			Self::Resized(physical_size) => {
				Event::Resized(Vec2::new(physical_size.width as f32, physical_size.height as f32))
			},
			Self::Ime(ime) => {
				match ime {
					winit::event::Ime::Commit(s) => Event::TextInput(s),
					winit::event::Ime::Enabled => Event::ImeEnable,
					winit::event::Ime::Disabled => Event::ImeDisable,
					_ => Event::NotSupported
				};
				Event::NotSupported
			},
			Self::MouseWheel{ delta, ..} => {
				match delta {
					winit::event::MouseScrollDelta::LineDelta(x, y) => Event::Scroll(Vec2::new(x, y) * 16.0),
					winit::event::MouseScrollDelta::PixelDelta(inner) => Event::Scroll(Vec2::new(inner.x as f32, inner.y as f32)),
				}
			},
			Self::Touch(touch) => {
				let location = Vec2::new(touch.location.x as f32, touch.location.y as f32);
				let id = touch.id as usize;
				match touch.phase {
					winit::event::TouchPhase::Started => Event::TouchStart(Touch {location, id, phase: TouchPhase::Start} ),
					winit::event::TouchPhase::Moved => Event::TouchStart(Touch {location, id, phase: TouchPhase::Move} ),
					winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled => Event::TouchStart(Touch {location, id, phase: TouchPhase::End} ),
				}
			},
			_ => { Event::NotSupported }
		}
	}
}

#[derive(Clone, Debug, PartialEq, Default)]
/// Human readable keyname which `nablo` foucus on
pub enum Key {
	#[default] A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z,
	Num0,Num1,Num2,Num3,Num4,Num5,Num6,Num7,Num8,Num9,NumPad1,NumPad2,NumPad3,NumPad4,NumPad5,NumPad6,NumPad7,NumPad8,NumPad9,NumPad0,NumLock,
	ArrowDown,ArrowLeft,ArrowRight,ArrowUp,Escape,Tab,Backquote,
	/// also contains number pad
	Backspace,Enter,Space,Insert,Delete,Home,End,PageUp,PageDown,PrintScreen,Minus,Equal,Pause,Period,ScrollLock,
	Backslash,BracketLeft,BracketRight,Comma,Slash,Semicolon,Quote,
	ControlLeft,ControlRight,CapsLock,ShiftLeft,ShiftRight,AltLeft,AltRight,
	F1,F2,F3,F4,F5,F6,F7,F8,F9,F10,F11,F12,F13,F14,F15,F16,F17,F18,F19,F20,F21,F22,F23,F24,F25,F26,F27,F28,F29,F30,F31,F32,F33,F34,F35,
	/// zero for unknown or unspported
	Unknown(usize)
}

impl Key {
	/// change a key to string
	pub fn to_string(&self, is_uppercase: bool) -> String {
		if !is_uppercase {
			match self {
				Key::A => "a",
				Key::B => "b",
				Key::C => "c",
				Key::D => "d",
				Key::E => "e",
				Key::F => "f",
				Key::G => "g",
				Key::H => "h",
				Key::I => "i",
				Key::J => "j",
				Key::K => "k",
				Key::L => "l",
				Key::M => "m",
				Key::N => "n",
				Key::O => "o",
				Key::P => "p",
				Key::Q => "q",
				Key::R => "r",
				Key::S => "s",
				Key::T => "t",
				Key::U => "u",
				Key::V => "v",
				Key::W => "w",
				Key::X => "x",
				Key::Y => "y",
				Key::Z => "z",
				Key::Num0 => "0",
				Key::Num1 => "1",
				Key::Num2 => "2",
				Key::Num3 => "3",
				Key::Num4 => "4",
				Key::Num5 => "5",
				Key::Num6 => "6",
				Key::Num7 => "7",
				Key::Num8 => "8",
				Key::Num9 => "9",
				Key::NumPad0 => "0",
				Key::NumPad1 => "1",
				Key::NumPad2 => "2",
				Key::NumPad3 => "3",
				Key::NumPad4 => "4",
				Key::NumPad5 => "5",
				Key::NumPad6 => "6",
				Key::NumPad7 => "7",
				Key::NumPad8 => "8",
				Key::NumPad9 => "9",
				Key::Backquote => "`",
				Key::Backslash => "\\",
				Key::BracketLeft => "[",
				Key::BracketRight => "]",
				Key::Comma => ",",
				Key::Space => " ",
				Key::Minus => "-",
				Key::Equal => "=",
				Key::Slash => "/",
				Key::Semicolon => ";",
				Key::Quote => "'",
				Key::Period => ".",
				_ => ""
			}.to_string()
		}else {
			match self {
				Key::A => "A",
				Key::B => "B",
				Key::C => "C",
				Key::D => "D",
				Key::E => "E",
				Key::F => "F",
				Key::G => "G",
				Key::H => "H",
				Key::I => "I",
				Key::J => "J",
				Key::K => "K",
				Key::L => "L",
				Key::M => "M",
				Key::N => "N",
				Key::O => "O",
				Key::P => "P",
				Key::Q => "Q",
				Key::R => "R",
				Key::S => "S",
				Key::T => "T",
				Key::U => "U",
				Key::V => "V",
				Key::W => "W",
				Key::X => "X",
				Key::Y => "Y",
				Key::Z => "Z",
				Key::Num0 => ")",
				Key::Num1 => "!",
				Key::Num2 => "@",
				Key::Num3 => "#",
				Key::Num4 => "$",
				Key::Num5 => "%",
				Key::Num6 => "^",
				Key::Num7 => "&",
				Key::Num8 => "*",
				Key::Num9 => "(",
				Key::Backquote => "~",
				Key::Backslash => "|",
				Key::BracketLeft => "{",
				Key::BracketRight => "}",
				Key::Comma => "<",
				Key::Space => " ",
				Key::Minus => "_",
				Key::Equal => "+",
				Key::Slash => "?",
				Key::Semicolon => ":",
				Key::Quote => "\"",
				Key::Period => ">",
				_ => ""
			}.to_string()
		}
		
	}
}

#[cfg(feature = "manager")]
impl Into<Key> for PhysicalKey {
	fn into(self) -> Key { 
		match self {
			Self::Code(known_key) => {
				match known_key {
					KeyCode::KeyA => Key::A,
					KeyCode::KeyB => Key::B,
					KeyCode::KeyC => Key::C,
					KeyCode::KeyD => Key::D,
					KeyCode::KeyE => Key::E,
					KeyCode::KeyF => Key::F,
					KeyCode::KeyG => Key::G,
					KeyCode::KeyH => Key::H,
					KeyCode::KeyI => Key::I,
					KeyCode::KeyJ => Key::J,
					KeyCode::KeyK => Key::K,
					KeyCode::KeyL => Key::L,
					KeyCode::KeyM => Key::M,
					KeyCode::KeyN => Key::N,
					KeyCode::KeyO => Key::O,
					KeyCode::KeyP => Key::P,
					KeyCode::KeyQ => Key::Q,
					KeyCode::KeyR => Key::R,
					KeyCode::KeyS => Key::S,
					KeyCode::KeyT => Key::T,
					KeyCode::KeyU => Key::U,
					KeyCode::KeyV => Key::V,
					KeyCode::KeyW => Key::W,
					KeyCode::KeyX => Key::X,
					KeyCode::KeyY => Key::Y,
					KeyCode::KeyZ => Key::Z,
					KeyCode::Digit0 => Key::Num0,
					KeyCode::Digit1 => Key::Num1,
					KeyCode::Digit2 => Key::Num2,
					KeyCode::Digit3 => Key::Num3,
					KeyCode::Digit4 => Key::Num4,
					KeyCode::Digit5 => Key::Num5,
					KeyCode::Digit6 => Key::Num6,
					KeyCode::Digit7 => Key::Num7,
					KeyCode::Digit8 => Key::Num8,
					KeyCode::Digit9 => Key::Num9,
					KeyCode::NumLock => Key::NumLock,
					KeyCode::Numpad0 => Key::NumPad0,
					KeyCode::Numpad1 => Key::NumPad1,
					KeyCode::Numpad2 => Key::NumPad2,
					KeyCode::Numpad3 => Key::NumPad3,
					KeyCode::Numpad4 => Key::NumPad4,
					KeyCode::Numpad5 => Key::NumPad5,
					KeyCode::Numpad6 => Key::NumPad6,
					KeyCode::Numpad7 => Key::NumPad7,
					KeyCode::Numpad8 => Key::NumPad8,
					KeyCode::Numpad9 => Key::NumPad9,
					KeyCode::F1 => Key::F1,
					KeyCode::F2 => Key::F2,
					KeyCode::F3 => Key::F3,
					KeyCode::F4 => Key::F4,
					KeyCode::F5 => Key::F5,
					KeyCode::F6 => Key::F6,
					KeyCode::F7 => Key::F7,
					KeyCode::F8 => Key::F8,
					KeyCode::F9 => Key::F9,
					KeyCode::F10 => Key::F10,
					KeyCode::F11 => Key::F11,
					KeyCode::F12 => Key::F12,
					KeyCode::F13 => Key::F13,
					KeyCode::F14 => Key::F14,
					KeyCode::F15 => Key::F15,
					KeyCode::F16 => Key::F16,
					KeyCode::F17 => Key::F17,
					KeyCode::F18 => Key::F18,
					KeyCode::F19 => Key::F19,
					KeyCode::F20 => Key::F20,
					KeyCode::F21 => Key::F21,
					KeyCode::F22 => Key::F22,
					KeyCode::F23 => Key::F23,
					KeyCode::F24 => Key::F24,
					KeyCode::F25 => Key::F25,
					KeyCode::F26 => Key::F26,
					KeyCode::F27 => Key::F27,
					KeyCode::F28 => Key::F28,
					KeyCode::F29 => Key::F29,
					KeyCode::F30 => Key::F30,
					KeyCode::F31 => Key::F31,
					KeyCode::F32 => Key::F32,
					KeyCode::F33 => Key::F33,
					KeyCode::F34 => Key::F34,
					KeyCode::F35 => Key::F35,
					KeyCode::Delete => Key::Delete,
					KeyCode::End => Key::End,
					KeyCode::Home => Key::Home,
					KeyCode::Insert => Key::Insert,
					KeyCode::PageDown => Key::PageDown,
					KeyCode::PageUp => Key::PageUp,
					KeyCode::ArrowDown => Key::ArrowDown,
					KeyCode::ArrowLeft => Key::ArrowLeft,
					KeyCode::ArrowRight => Key::ArrowRight,
					KeyCode::ArrowUp => Key::ArrowUp,
					KeyCode::Backquote => Key::Backquote,
					KeyCode::Backslash => Key::Backslash,
					KeyCode::BracketLeft => Key::BracketLeft,
					KeyCode::BracketRight => Key::BracketRight,
					KeyCode::Comma => Key::Comma,
					KeyCode::Backspace | KeyCode::NumpadBackspace => Key::Backspace,
					KeyCode::ControlLeft => Key::ControlLeft,
					KeyCode::ControlRight => Key::ControlRight,
					KeyCode::Escape => Key::Escape,
					KeyCode::Tab => Key::Tab,
					KeyCode::Enter => Key::Enter,
					KeyCode::Space => Key::Space,
					KeyCode::Minus => Key::Minus,
					KeyCode::Pause => Key::Pause,
					KeyCode::Equal => Key::Equal,
					KeyCode::CapsLock => Key::CapsLock,
					KeyCode::ShiftLeft => Key::ShiftLeft,
					KeyCode::ShiftRight => Key::ShiftRight,
					KeyCode::Slash => Key::Slash,
					KeyCode::Semicolon => Key::Semicolon,
					KeyCode::Quote => Key::Quote,
					KeyCode::AltLeft => Key::AltLeft,
					KeyCode::AltRight => Key::AltRight,
					KeyCode::Period => Key::Period,
					KeyCode::PrintScreen => Key::PrintScreen,
					KeyCode::ScrollLock => Key::ScrollLock,
					_ => {Key::Unknown(0)},
				}
			},
			Self::Unidentified(code) => {
				match code {
					winit::keyboard::NativeKeyCode::Unidentified => Key::Unknown(0),
					winit::keyboard::NativeKeyCode::Android(t) => Key::Unknown(t as usize),
					winit::keyboard::NativeKeyCode::MacOS(t) => Key::Unknown(t as usize),
					winit::keyboard::NativeKeyCode::Windows(t) => Key::Unknown(t as usize),
					winit::keyboard::NativeKeyCode::Xkb(t) => Key::Unknown(t as usize),
				}
			}
		}
	}
}
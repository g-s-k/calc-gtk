use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use gdk::EventType::KeyPress;
use gtk::{prelude::*, Button, Grid, Window, WindowType};

macro_rules! update_disp {
    ( $state:expr, $displ:expr ) => {
        $displ.set_text(&format!("{}", $state.borrow()))
    };
}

macro_rules! btn {
    ( $num:expr, $state:expr, $displ:expr ) => {{
        let btn = Button::new_with_label($num);
        let state_c = $state.clone();
        let displ_c = $displ.clone();
        btn.set_can_focus(false);
        btn.connect_clicked(move |_| {
            state_c.borrow_mut().arg.push_str($num);
            update_disp!(state_c, displ_c);
        });
        btn
    }};

    ( $symb:expr, $op:expr, $state:expr, $displ:expr ) => {{
        let btn = Button::new_with_label($symb);
        let state_c = $state.clone();
        let displ_c = $displ.clone();
        btn.set_can_focus(false);
        btn.connect_clicked(move |_| {
            state_c.borrow_mut().exec();
            state_c.borrow_mut().op = Some($op);
            update_disp!(state_c, displ_c);
        });
        btn
    }};
}

#[derive(Clone, Copy, Debug)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn to_fn(self) -> fn(f64, f64) -> f64 {
        match self {
            Op::Add => std::ops::Add::add,
            Op::Sub => std::ops::Sub::sub,
            Op::Mul => std::ops::Mul::mul,
            Op::Div => std::ops::Div::div,
        }
    }
}

#[derive(Debug, Default)]
struct State {
    current: Option<f64>,
    arg: String,
    inv: bool,
    op: Option<Op>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.arg.is_empty() {
            if let Some(c) = self.current {
                // argh rust doesn't have a `g` format type
                let e = format!("{:.6e}", c);
                let g = format!("{}", c);
                return write!(f, "{}", if e.len() < g.len() { e } else { g });
            } else {
                return Ok(());
            }
        }

        if self.inv {
            write!(f, "-")?;
        }

        write!(f, "{}", self.arg)
    }
}

impl State {
    fn clear(&mut self) {
        *self = Default::default();
    }

    fn div_100(&mut self) {
        self.exec();
        if let Some(c) = self.current.iter_mut().next() {
            *c /= 100.0;
        }
    }

    fn get_val(&mut self) -> f64 {
        if self.arg.is_empty() {
            return 0.0;
        }

        std::mem::replace(&mut self.arg, String::new())
            .parse::<f64>()
            .expect("Could not parse as float")
            * if self.inv {
                self.inv = false;
                -1.0
            } else {
                1.0
            }
    }

    fn exec(&mut self) {
        match (self.current, self.op.take()) {
            (None, _) | (_, None) => {
                if !self.arg.is_empty() {
                    self.current = Some(self.get_val());
                }
            }
            (Some(c), Some(op)) => {
                self.current = Some(op.to_fn()(c, self.get_val()));
            }
        }
    }
}

fn build_ui() -> Window {
    let ctr = Rc::new(RefCell::new(State::default()));
    let out = gtk::Label::new(None);
    out.set_halign(gtk::Align::End);
    out.set_property_margin(5);

    let btn_grid = Grid::new();
    btn_grid.attach(&out, 0, 0, 4, 1);

    // clear button
    let button_clr = Button::new_with_label("AC");
    btn_grid.attach(&button_clr, 0, 1, 1, 1);
    let ctrc = ctr.clone();
    let outc = out.clone();
    button_clr.set_can_focus(false);
    button_clr.connect_clicked(move |_| {
        ctrc.borrow_mut().clear();
        update_disp!(ctrc, outc);
    });

    // sign inversion button
    let button_inv = Button::new_with_label("±");
    btn_grid.attach(&button_inv, 1, 1, 1, 1);
    let ctrc = ctr.clone();
    let outc = out.clone();
    button_inv.set_can_focus(false);
    button_inv.connect_clicked(move |_| {
        ctrc.borrow_mut().inv ^= true;
        update_disp!(ctrc, outc);
    });

    let button_pct = Button::new_with_label("%");
    btn_grid.attach(&button_pct, 2, 1, 1, 1);
    let ctrc = ctr.clone();
    let outc = out.clone();
    button_pct.set_can_focus(false);
    button_pct.connect_clicked(move |_| {
        ctrc.borrow_mut().div_100();
        update_disp!(ctrc, outc);
    });

    btn_grid.attach(&btn!("÷", Op::Div, ctr, out), 3, 1, 1, 1);
    btn_grid.attach(&btn!("7", ctr, out), 0, 2, 1, 1);
    btn_grid.attach(&btn!("8", ctr, out), 1, 2, 1, 1);
    btn_grid.attach(&btn!("9", ctr, out), 2, 2, 1, 1);
    btn_grid.attach(&btn!("×", Op::Mul, ctr, out), 3, 2, 1, 1);
    btn_grid.attach(&btn!("4", ctr, out), 0, 3, 1, 1);
    btn_grid.attach(&btn!("5", ctr, out), 1, 3, 1, 1);
    btn_grid.attach(&btn!("6", ctr, out), 2, 3, 1, 1);
    btn_grid.attach(&btn!("-", Op::Sub, ctr, out), 3, 3, 1, 1);
    btn_grid.attach(&btn!("1", ctr, out), 0, 4, 1, 1);
    btn_grid.attach(&btn!("2", ctr, out), 1, 4, 1, 1);
    btn_grid.attach(&btn!("3", ctr, out), 2, 4, 1, 1);
    btn_grid.attach(&btn!("+", Op::Add, ctr, out), 3, 4, 1, 1);
    btn_grid.attach(&btn!("0", ctr, out), 0, 5, 2, 1);

    // decimal point
    let button_pt = Button::new_with_label(".");
    btn_grid.attach(&button_pt, 2, 5, 1, 1);
    let ctrc = ctr.clone();
    let outc = out.clone();
    button_pt.set_can_focus(false);
    button_pt.connect_clicked(move |_| {
        if ctrc.borrow().arg.contains('.') {
            return;
        }

        ctrc.borrow_mut().arg.push('.');
        update_disp!(ctrc, outc);
    });

    let button_eq = Button::new_with_label("=");
    btn_grid.attach(&button_eq, 3, 5, 1, 1);
    let ctrc = ctr.clone();
    let outc = out.clone();
    button_eq.grab_focus();
    button_eq.connect_clicked(move |_| {
        ctrc.borrow_mut().exec();
        update_disp!(ctrc, outc);
    });

    let window = Window::new(WindowType::Toplevel);
    window.set_title("calculator");
    window.set_resizable(false);
    window.add(&btn_grid);
    let ctrc = ctr.clone();
    let outc = out.clone();
    window.connect_event(move |w, e| {
        if let KeyPress = e.get_event_type() {
            match e.get_keyval().and_then(gdk::keyval_to_unicode) {
                Some(n @ '0'...'9') => {
                    ctrc.borrow_mut().arg.push(n);
                }
                Some('.') => {
                    if !ctrc.borrow().arg.contains('.') {
                        ctrc.borrow_mut().arg.push('.');
                    }
                }
                Some(o) if o == '+' || o == '-' || o == '*' || o == '/' => {
                    ctrc.borrow_mut().exec();
                    ctrc.borrow_mut().op = Some(match o {
                        '+' => Op::Add,
                        '-' => Op::Sub,
                        '*' => Op::Mul,
                        '/' => Op::Div,
                        _ => unreachable!(),
                    });
                }
                Some(' ') | Some('\t') => {
                    ctrc.borrow_mut().clear();
                }
                Some('i') => {
                    ctrc.borrow_mut().inv ^= true;
                }
                Some('p') => {
                    ctrc.borrow_mut().div_100();
                }
                Some('Q') => {
                    w.close();
                }
                _ => return Inhibit(false),
            }
            update_disp!(ctrc, outc);
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    });
    window
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let window = build_ui();
    window.show_all();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}

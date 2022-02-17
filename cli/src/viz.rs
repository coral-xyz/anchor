#![allow(unused_variables, unused_imports)]
use anchor_syn::idl::{Idl, IdlAccountItem, IdlAccount, IdlAccounts};
use plotters::prelude::*;
use plotters::style::ShapeStyle;
use plotters::style::text_anchor::Pos;
use plotters_backend::text_anchor::{HPos, VPos};
use plotters_backend::{FontStyle, BackendColor};
use std::convert::TryInto;


pub fn visualize(idl: Idl, viz_args: Vec<String>, out: &str) {

    // how many items per row, per instruction/method.
    const ROW_WIDTH : usize = 2;
    // width and height of fig objects
    const BOX_PX_WIDTH : usize = 240;
    const BOX_PX_HEIGHT : usize = 60;
    // width of header for title
    const HEADER_PX_HEIGHT : usize = 100;
    // width of vertical separator
    const SEP_WIDTH : usize = 2; 
    // vertical and horizontal size of gap between objects 
    const BUFFER_WIDTH : usize = 8;
    // size of title and other text
    const TITLE_SIZE : i32 = 24;
    const TEXT_SIZE : i32 = 20;

    // Find width and height of figure
    // width: total columns = instructions + state methods
    let mut state_methods = match idl.state.clone() {
        Some(idlstate) => idlstate.methods,
        None => vec![],
    };
    let state_name = match idl.state.clone() {
        Some(idlstate) => idlstate.strct.name,
        None => "".to_string(),
    };
    let columns = idl.instructions.len() + state_methods.len();

    // height: Initialize tracker to find largest instruction/state_method
    let mut rows = 0;
    let mut all_instructions = idl.instructions.clone();
    all_instructions.append(&mut state_methods.clone());
    for instruction in all_instructions {

        // count all accounts not in groups
        let accounts = unpack_group(IdlAccounts {
            name: "".to_string(), 
            accounts: instruction.accounts.clone() 
        });

        let num_accounts = accounts.len();
        let acct_height = {
            if num_accounts % ROW_WIDTH == 0 {
                num_accounts / ROW_WIDTH
            } else{
                num_accounts / ROW_WIDTH + 1
            }
        };

        // count all signers
        // instruction + group signers
        let signers = accounts
            .iter()
            .fold(0, |acc, x| acc + if x.is_signer { 1 } else { 0 });

        let sign_height = {
            if signers % ROW_WIDTH == 0 {
                signers / ROW_WIDTH
            } else {
                signers / ROW_WIDTH + 1
            }
        }.max(1);

        let args = instruction.args.len();
        let arg_height = {
            if args % ROW_WIDTH == 0 {
                args / ROW_WIDTH
            } else {
                args / ROW_WIDTH + 1
            }
        };

        let height = arg_height + acct_height + sign_height;
        rows = rows.max(height);
    }


    // Steps to take
    // 0) Create a canvas to draw on
    // 1) Title and version
    // 2) Populate vertical separator lines
    // 3) Populate anchor instruction names
    // 4) Populate mut accts
    // 5) Populate immut accounts
    // 6) Populate signers
    // 7) populate args

    // 0) Create a canvas to draw on
    let fig_width : u32 = ((BOX_PX_WIDTH + BUFFER_WIDTH) * ROW_WIDTH * columns + BUFFER_WIDTH*columns + (columns-1)*SEP_WIDTH).try_into().unwrap();
    let fig_height : u32 = ((BOX_PX_HEIGHT + BUFFER_WIDTH) * rows + HEADER_PX_HEIGHT + 3*BUFFER_WIDTH + BOX_PX_HEIGHT + 2*BUFFER_WIDTH).try_into().unwrap();
    let backend = BitMapBackend::new(out, (fig_width, fig_height)).into_drawing_area();
    backend.fill(&WHITE).expect("couldn't fill background color");

    // 1) Title and version
    backend.draw(
        &Text::new(
            format!("Anchor Program: {}", idl.name),
            (fig_width as i32/2, HEADER_PX_HEIGHT as i32/4),
            TextStyle {
                font: FontDesc::new(
                    FontFamily::Monospace,
                    TITLE_SIZE as f64,
                    FontStyle::Bold),
                color: BackendColor { 
                    alpha: 1.0, 
                    rgb: (0,0,0),
                },
                pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
            }
        )
    ).expect("couldn't write 'Anchor Program'");
    backend.draw(
        &Text::new(
            format!("Version: {}", idl.version),
            (fig_width as i32/2, HEADER_PX_HEIGHT as i32/2),
            TextStyle {
                font: FontDesc::new(
                    FontFamily::Monospace,
                    TITLE_SIZE as f64,
                    FontStyle::Normal),
                color: BackendColor { 
                    alpha: 1.0, 
                    rgb: (0,0,0),
                },
                pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
            }
        )
    ).expect("couldn't write version");

    // 2) Vertical Separator lines
    for i in 1..columns {

        // Thin Rectangles as vertical separators
        backend.draw(
            &Rectangle::new(
            
                [(
                    // top left
                    ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i-SEP_WIDTH) as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH) as i32,
                ),(
                    // bottom right
                    ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i) as i32,
                    // ((rows+1)*BOX_PX_HEIGHT + HEADER_PX_HEIGHT + (rows + 2)*BUFFER_WIDTH) as i32, // idk why this doesn't work... this is (in principle) fig_height - 3*BUFFER_WIDTH
                    (fig_height as i32 -3*BUFFER_WIDTH as i32),
                )], Into::<ShapeStyle>::into(&BLACK).filled()
            )
        ).expect("couldn't draw vertical separators");
    }

    // 3) Populate instruction + state method names
    // 4) Populate signers
    // 5) Populate mut accts
    // 6) Populate immut accounts
    // 7) populate args

    // 3) Populate instruction + state method names
    for (i, instruction) in idl.instructions.iter().enumerate() {
        backend.draw(
            &Rectangle::new(
            
                [(
                    // top left
                    ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + (BOX_PX_WIDTH*ROW_WIDTH + BUFFER_WIDTH*(ROW_WIDTH+1))/2 - BOX_PX_WIDTH/2) as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH) as i32,
                ),(
                    // bottom right
                    ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + (BOX_PX_WIDTH*ROW_WIDTH + BUFFER_WIDTH*(ROW_WIDTH+1))/2 + BOX_PX_WIDTH/2) as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH + BOX_PX_HEIGHT) as i32,
                )], Into::<ShapeStyle>::into(&RGBColor(255,200,200)).filled()
            )
        ).expect("couldn't draw rect for instruction");
        backend.draw(
            &Text::new(
                format!("Instruction:"),
                (
                    ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH + (BOX_PX_WIDTH*ROW_WIDTH + BUFFER_WIDTH*(ROW_WIDTH+1))/2) as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH + BOX_PX_HEIGHT/3) as i32),
                TextStyle {
                    font: FontDesc::new(
                        FontFamily::Monospace,
                        TEXT_SIZE as f64,
                        FontStyle::Normal),
                    color: BackendColor { 
                        alpha: 1.0, 
                        rgb: (0,0,0),
                    },
                    pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                }
            )
        ).expect("couldn't write 'Instruction'");
        backend.draw(
            &Text::new(
                format!("{}", instruction.name),
                (
                    ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH + (BOX_PX_WIDTH*ROW_WIDTH + BUFFER_WIDTH*(ROW_WIDTH+1))/2) as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH + 2*BOX_PX_HEIGHT/3) as i32),
                TextStyle {
                    font: FontDesc::new(
                        FontFamily::Monospace,
                        TEXT_SIZE as f64,
                        FontStyle::Normal),
                    color: BackendColor { 
                        alpha: 1.0, 
                        rgb: (0,0,0),
                    },
                    pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                }
            )
        ).expect("couldn't write instruction name");
    }
    for (i, state_method) in state_methods.iter().enumerate() {
        let i = i + idl.instructions.len();
        backend.draw(
            &Rectangle::new(
            
                [(
                    // top left
                    ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + (BOX_PX_WIDTH*ROW_WIDTH + BUFFER_WIDTH*(ROW_WIDTH+1))/2 - BOX_PX_WIDTH/2) as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH) as i32,
                ),(
                    // bottom right
                    ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + (BOX_PX_WIDTH*ROW_WIDTH + BUFFER_WIDTH*(ROW_WIDTH+1))/2 + BOX_PX_WIDTH/2) as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH + BOX_PX_HEIGHT) as i32,
                )], Into::<ShapeStyle>::into(&RGBColor(255,200,200)).filled()
            )
        ).expect("couldn't draw rect for instruction");
        backend.draw(
            &Text::new(
                format!("State Method:"),
                (
                    ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH + (BOX_PX_WIDTH*ROW_WIDTH + BUFFER_WIDTH*(ROW_WIDTH+1))/2) as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH + BOX_PX_HEIGHT/3) as i32),
                TextStyle {
                    font: FontDesc::new(
                        FontFamily::Monospace,
                        TEXT_SIZE as f64,
                        FontStyle::Normal),
                    color: BackendColor { 
                        alpha: 1.0, 
                        rgb: (0,0,0),
                    },
                    pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                }
            )
        ).expect("couldn't write 'State Method'");
        backend.draw(
            &Text::new(
                format!("{}.{}", state_name, state_method.name),
                (
                    ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH + (BOX_PX_WIDTH*ROW_WIDTH + BUFFER_WIDTH*(ROW_WIDTH+1))/2) as i32,
                    (HEADER_PX_HEIGHT + BUFFER_WIDTH + 2*BOX_PX_HEIGHT/3) as i32),
                TextStyle {
                    font: FontDesc::new(
                        FontFamily::Monospace,
                        TEXT_SIZE as f64,
                        FontStyle::Normal),
                    color: BackendColor { 
                        alpha: 1.0, 
                        rgb: (0,0,0),
                    },
                    pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                }
            )
        ).expect("couldn't write state method name");
    }

    
    
    // concat all instructions + methods
    let mut all_instructions = idl.instructions.clone();
    all_instructions.append(&mut state_methods);
    for (i, instruction) in all_instructions.iter().enumerate() {

        let accounts = unpack_group(IdlAccounts {
            name: "".to_string(), 
            accounts: instruction.accounts.clone() 
        });

        let inst_signers : Vec<&IdlAccount> = accounts.iter().filter(|&x| x.is_signer).collect();
        let mut signers = 0; // counter
        // 4) Populate signers
        for &signer in inst_signers.iter() {
            
            let (l, k) = (signers / ROW_WIDTH, signers % ROW_WIDTH);

            backend.draw(
                &Rectangle::new(
                
                    [(
                        // top left
                        ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k) as i32,
                        (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BOX_PX_HEIGHT + BUFFER_WIDTH*(1+l) + BOX_PX_HEIGHT*(l)) as i32,
                    ),(
                        // bottom right
                        ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*(k+1)) as i32,
                        (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BOX_PX_HEIGHT + BUFFER_WIDTH*(1+l) + BOX_PX_HEIGHT*(l+1)) as i32,
                    )], Into::<ShapeStyle>::into(&RGBColor(0,255,163)).filled()
                )
            ).expect("couldn't draw rect for signer");
            backend.draw(
                &Text::new(
                    format!("Signer:"),
                    (
                        ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k + BOX_PX_WIDTH/2) as i32,
                        (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BUFFER_WIDTH*(1+l) + BOX_PX_HEIGHT*(l+1)+BOX_PX_HEIGHT/3) as i32,
                    ),
                    TextStyle {
                        font: FontDesc::new(
                            FontFamily::Monospace,
                            TEXT_SIZE as f64,
                            FontStyle::Normal),
                        color: BackendColor { 
                            alpha: 1.0, 
                            rgb: (0,0,0),
                        },
                        pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                    }
                )
            ).expect("couldn't write 'Signer:'");
            backend.draw(
                &Text::new(
                    format!("{}", signer.name),
                    (
                        ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k + BOX_PX_WIDTH/2) as i32,
                        (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BUFFER_WIDTH*(1+l) + BOX_PX_HEIGHT*(l+1)+2*BOX_PX_HEIGHT/3) as i32,
                    ),
                    TextStyle {
                        font: FontDesc::new(
                            FontFamily::Monospace,
                            TEXT_SIZE as f64,
                            FontStyle::Normal),
                        color: BackendColor { 
                            alpha: 1.0, 
                            rgb: (0,0,0),
                        },
                        pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                    }
                )
            ).expect("couldn't write signer");

            signers += 1;
        }

        
        let signer_offset = {
            if signers % ROW_WIDTH == 0 {
                signers / ROW_WIDTH
            } else {
                signers / ROW_WIDTH + 1
            }.max(1)
        };

        let mut accounts_drawn = 0;

        // 5) Populate mut accts
        for account in accounts.clone() {
            
            if account.is_mut{

                let (l, k) = (accounts_drawn / ROW_WIDTH, accounts_drawn % ROW_WIDTH);

                backend.draw(
                    &Rectangle::new(
                    
                        [(
                            // top left
                            ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k) as i32,
                            (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BOX_PX_HEIGHT + BUFFER_WIDTH*(1+signer_offset+l) + BOX_PX_HEIGHT*(l+signer_offset)) as i32,
                        ),(
                            // bottom right
                            ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*(k+1)) as i32,
                            (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BOX_PX_HEIGHT + BUFFER_WIDTH*(1+signer_offset+l) + BOX_PX_HEIGHT*(l+1+signer_offset)) as i32,
                        )], Into::<ShapeStyle>::into(&RGBColor(255,100,100)).filled()
                    )
                ).expect("couldn't draw rect for mutable account");
                backend.draw(
                    &Text::new(
                        format!("Mutable Account:"),
                        (
                            ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k + BOX_PX_WIDTH/2) as i32,
                            (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BUFFER_WIDTH*(signer_offset+l) + BOX_PX_HEIGHT*(l+1+signer_offset)+BOX_PX_HEIGHT/3) as i32,
                        ),
                        TextStyle {
                            font: FontDesc::new(
                                FontFamily::Monospace,
                                TEXT_SIZE as f64,
                                FontStyle::Normal),
                            color: BackendColor { 
                                alpha: 1.0, 
                                rgb: (0,0,0),
                            },
                            pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                        }
                    )
                ).expect("couldn't write 'Mutable Account:'");
                backend.draw(
                    &Text::new(
                        format!("{}", account.name),
                        (
                            ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k + BOX_PX_WIDTH/2) as i32,
                            (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BUFFER_WIDTH*(1+signer_offset+l) + BOX_PX_HEIGHT*(l+1+signer_offset)+2*BOX_PX_HEIGHT/3) as i32,
                        ),
                        TextStyle {
                            font: FontDesc::new(
                                FontFamily::Monospace,
                                TEXT_SIZE as f64,
                                FontStyle::Normal),
                            color: BackendColor { 
                                alpha: 1.0, 
                                rgb: (0,0,0),
                            },
                            pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                        }
                    )
                ).expect("couldn't write mut account name");

                accounts_drawn += 1;
            }
        }

        // 6) Populate immut accts
        for account in &accounts.clone() {

            if !account.is_mut{

                let (l, k) = (accounts_drawn / ROW_WIDTH, accounts_drawn % ROW_WIDTH);

                backend.draw(
                    &Rectangle::new(
                    
                        [(
                            // top left
                            ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k) as i32,
                            (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BOX_PX_HEIGHT + BUFFER_WIDTH*(1+signer_offset+l) + BOX_PX_HEIGHT*(l+signer_offset)) as i32,
                        ),(
                            // bottom right
                            ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*(k+1)) as i32,
                            (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BOX_PX_HEIGHT + BUFFER_WIDTH*(1+signer_offset+l) + BOX_PX_HEIGHT*(l+signer_offset+1)) as i32,
                        )], Into::<ShapeStyle>::into(&RGBColor(3,225,255)).filled()
                    )
                ).expect("couldn't draw rect for immutable account");
                backend.draw(
                    &Text::new(
                        format!("Immutable Account:"),
                        (
                            ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k + BOX_PX_WIDTH/2) as i32,
                            (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BUFFER_WIDTH*(1+signer_offset+l) + BOX_PX_HEIGHT*(l+1+signer_offset)+BOX_PX_HEIGHT/3) as i32,
                        ),
                        TextStyle {
                            font: FontDesc::new(
                                FontFamily::Monospace,
                                TEXT_SIZE as f64,
                                FontStyle::Normal),
                            color: BackendColor { 
                                alpha: 1.0, 
                                rgb: (0,0,0),
                            },
                            pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                        }
                    )
                ).expect("couldn't write 'Immutable Account:'");
                backend.draw(
                    &Text::new(
                        format!("{}", account.name),
                        (
                            ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k + BOX_PX_WIDTH/2) as i32,
                            (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BUFFER_WIDTH*(1+signer_offset+l) + BOX_PX_HEIGHT*(l+1+signer_offset)+2*BOX_PX_HEIGHT/3) as i32,
                        ),
                        TextStyle {
                            font: FontDesc::new(
                                FontFamily::Monospace,
                                TEXT_SIZE as f64,
                                FontStyle::Normal),
                            color: BackendColor { 
                                alpha: 1.0, 
                                rgb: (0,0,0),
                            },
                            pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                        }
                    )
                ).expect("couldn't write immut account name");

                accounts_drawn += 1;
            }
        }
    
        
        let account_offset = {
            if accounts_drawn % ROW_WIDTH == 0 {
                accounts_drawn / ROW_WIDTH
            } else {
                accounts_drawn / ROW_WIDTH + 1
            }
        };
        let offset = signer_offset + account_offset;

        let mut args_drawn = 0;

        // 7) Populate args
        for arg in instruction.args.iter() {
    
            let (l, k) = (args_drawn / ROW_WIDTH, args_drawn % ROW_WIDTH);

            backend.draw(
                &Rectangle::new(
                
                    [(
                        // top left
                        ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k) as i32,
                        (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BOX_PX_HEIGHT + BUFFER_WIDTH*(1+offset+l) + BOX_PX_HEIGHT*(l+offset)) as i32,
                    ),(
                        // bottom right
                        ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*(k+1)) as i32,
                        (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BOX_PX_HEIGHT + BUFFER_WIDTH*(1+offset+l) + BOX_PX_HEIGHT*(l+offset+1)) as i32,
                    )], Into::<ShapeStyle>::into(&RGBColor(220,31,255)).filled()
                )
            ).expect("couldn't draw rect for argument");
            backend.draw(
                &Text::new(
                    format!("Argument:"),
                    (
                        ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k + BOX_PX_WIDTH/2) as i32,
                        (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BUFFER_WIDTH*(1+offset+l) + BOX_PX_HEIGHT*(l+1+offset)+BOX_PX_HEIGHT/3) as i32,
                    ),
                    TextStyle {
                        font: FontDesc::new(
                            FontFamily::Monospace,
                            TEXT_SIZE as f64,
                            FontStyle::Normal),
                        color: BackendColor { 
                            alpha: 1.0, 
                            rgb: (0,0,0),
                        },
                        pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                    }
                )
            ).expect("couldn't write 'Argument:'");
            backend.draw(
                &Text::new(
                    format!("{}", arg.name),
                    (
                        ((BOX_PX_WIDTH*ROW_WIDTH + SEP_WIDTH + (1+ROW_WIDTH)*BUFFER_WIDTH)*i + BUFFER_WIDTH*(k+1) + BOX_PX_WIDTH*k + BOX_PX_WIDTH/2) as i32,
                        (HEADER_PX_HEIGHT + 2*BUFFER_WIDTH + BUFFER_WIDTH*(1+offset+l) + BOX_PX_HEIGHT*(l+1+offset)+2*BOX_PX_HEIGHT/3) as i32,
                    ),
                    TextStyle {
                        font: FontDesc::new(
                            FontFamily::Monospace,
                            TEXT_SIZE as f64,
                            FontStyle::Normal),
                        color: BackendColor { 
                            alpha: 1.0, 
                            rgb: (0,0,0),
                        },
                        pos: Pos { h_pos: HPos::Center, v_pos: VPos::Center },
                    }
                )
            ).expect("couldn't write argument");

            args_drawn += 1;
        }
    }
}

fn unpack_group(account_group: IdlAccounts) -> Vec<IdlAccount>{

    let mut v : Vec<IdlAccount> = vec![];

    for account in account_group.accounts.iter() { 

        match account { 
            IdlAccountItem::IdlAccount(idl_account) => v.push(idl_account.clone()),
            IdlAccountItem::IdlAccounts(idl_accounts) => v.append(&mut unpack_group(idl_accounts.clone())),
        };
    }
    v
}
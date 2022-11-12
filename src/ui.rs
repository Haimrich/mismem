use crate::app::App;
use crate::app::AppState;
use crate::app::EditState;

use tui::Frame;
use tui::backend::Backend;
use tui::layout::{Layout,Constraint,Alignment,Direction,Rect};
use tui::style::{Color,Modifier,Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Table,Row,Cell,Block,Borders,Paragraph,Clear,Gauge,List,ListItem};

use tui_logger::{TuiLoggerWidget,TuiLoggerLevelOutput};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    match app.state {
        AppState::Home => {},
        AppState::SelectProcess => draw_select_process(f, app),
        AppState::EditMemory => draw_edit_memory(f, app)
    };
}

fn draw_select_process<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let main_height = size.height.checked_sub(4).unwrap_or_default();

    let rects = Layout::default()
        .constraints([
            Constraint::Length(1),
            Constraint::Length(main_height),
            Constraint::Length(1),
        ].as_ref())
        .margin(1)
        .split(size);

    // Top Messages
    let msg = Text::from("Select a running process");
    let top_message = Paragraph::new(msg).alignment(Alignment::Center);
    f.render_widget(top_message, rects[0]);

    // Help
    let msg = vec![
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit | "),
        Span::styled("u", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" update | "),
        Span::styled("▲", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("/"),
        Span::styled("▼", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" navigate | "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" select"),
    ];
    let msg = Text::from(Spans::from(msg));
    
    let help_message = Paragraph::new(msg).alignment(Alignment::Center);
    f.render_widget(help_message, rects[2]);

    // Process List
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_cells = ["PID", "Process Name", "Memory Usage [kB]"]
        .iter()
        .map( |h| Cell::from(*h) );
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray).fg(Color::Black))
        .height(1)
        .bottom_margin(1);
    let rows = app.processes.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.clone()));
        Row::new(cells).height(height as u16)
    });
    let t = Table::new(rows)
        .header(header)
        .column_spacing(1)
        .block(Block::default().borders(Borders::ALL).title("Process List"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Length(8),
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ]);
    f.render_stateful_widget(t, rects[1], &mut app.table_state);

    // Error Popup
    if app.show_popup {
        let area = centered_rect(60, 20, size);
        
        let block = Block::default().title("Error").title_alignment(Alignment::Center).borders(Borders::ALL).style(Style::default().fg(Color::Yellow));
        
        let msg = Text::from("Error selecting this process.");
        let msg = Paragraph::new(msg).alignment(Alignment::Center);
        
        let rects = Layout::default()
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(block.inner(area));
            
        f.render_widget(Clear, area);
        f.render_widget(block, area);
        f.render_widget(msg, rects[1]);
    }
}


fn draw_edit_memory<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let main_height = size.height.checked_sub(4).unwrap_or_default();

    let rects = Layout::default()
        .constraints([
            Constraint::Length(1),
            Constraint::Length(main_height),
            Constraint::Length(1),
        ].as_ref())
        .margin(1)
        .split(f.size());

    // Top Messages
    let msg = vec![
        Span::raw("Process "),
        Span::raw(app.selected_process.to_string()),
    ];
    let msg = Text::from(Spans::from(msg));
    
    let top_message = Paragraph::new(msg).alignment(Alignment::Center);

    f.render_widget(top_message, rects[0]);


    // Help
    let msg = vec![
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit | "),
        Span::styled("u", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" update | "),
        Span::styled("◄", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" back | "),
        Span::styled("▲", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("/"),
        Span::styled("▼", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" navigate | "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" select"),
    ];
    let msg = Text::from(Spans::from(msg));
    
    let help_message = Paragraph::new(msg).alignment(Alignment::Center);
    f.render_widget(help_message, rects[2]);

    
    // Main Part
    let rects = Layout::default()
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50)
        ].as_ref())
        .direction(Direction::Horizontal)
        .split(rects[1]);

    // Memory List
    
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_cells = ["Address", "Value", "Old Value"]
        .iter()
        .map( |h| Cell::from(*h) );
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray).fg(Color::Black))
        .height(1)
        .bottom_margin(1);

    // Eager loading in chunks bacause of too many rows, this only works in one direction, TODO extend to both directions one day (never)
    const EAGER_CHUNK_SIZE : usize = 40;
    let num_rows_to_load = (app.table_state.selected().unwrap_or(0) / EAGER_CHUNK_SIZE + 2) * EAGER_CHUNK_SIZE;
    
    let rows = app.memory.iter().take(num_rows_to_load).map(|item| {
        let cells = item.iter().map(|c| Cell::from(c.clone()));
        Row::new(cells)
    });
   
    let t = Table::new(rows)
        .header(header)
        .column_spacing(1)
        .block(Block::default().borders(Borders::ALL)
            .title(" 💾 Results ")
            .style(if matches!(app.edit_state, EditState::Select) && !app.show_popup {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.table_state);

    
    // Panel
    let rects = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Length(3),
            Constraint::Percentage(40)
        ].as_ref())
        .direction(Direction::Vertical)
        .split(rects[1]);
    
    let width = rects[1].width.max(3) - 3;
    let scroll = (app.search_input.cursor() as u16).max(width) - width;
    let input = Paragraph::new(app.search_input.value())
        .style(if matches!(app.edit_state, EditState::Input) && !app.show_popup {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        })
        .scroll((0, scroll))
        .block(Block::default().borders(Borders::ALL).title(" 🔎 Search ").title_alignment(Alignment::Center));
    f.render_widget(input, rects[0]);
    if matches!(app.edit_state, EditState::Input) && !app.show_popup {
        f.set_cursor(
            rects[0].x + (app.search_input.cursor() as u16).min(width) + 1,
            rects[0].y + 1,
        )
    }

    // Progress Gauge
    let label = format!("{:.1}%", app.search_progress * 100.0);
    let gauge = Gauge::default()
        .block(Block::default().title(" 🚀 Search Progress ")
        .borders(Borders::ALL)
        .style(if matches!(app.edit_state, EditState::Busy) && !app.show_popup {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        }))
        .gauge_style(if matches!(app.edit_state, EditState::Busy) && !app.show_popup {
            Style::default().fg(Color::Yellow).bg(Color::Reset)
        } else {
            Style::default().fg(Color::DarkGray).bg(Color::Reset)
        })
        .ratio(app.search_progress)
        .label(label);
    f.render_widget(gauge, rects[2]);

    // Logs
    let tui_w: TuiLoggerWidget = TuiLoggerWidget::default()
        .block(
            Block::default().title(" 📜 Logs ").borders(Borders::ALL)
        )
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Cyan))
        .style_info(Style::default().fg(Color::White))
        .output_separator(':')
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .style(Style::default().fg(Color::White).bg(Color::Reset));
    f.render_widget(tui_w, rects[3]);

    // Search Settings
    let rects = Layout::default()
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ].as_ref())
        .direction(Direction::Horizontal)
        .split(rects[1]);


    fn create_opt_list<'a>(opts : &'a [&'a str], key : &'a str, name : &'a str) -> List<'a> {
        let options : Vec<ListItem> = opts.iter().map(|o| ListItem::new(Text::raw(o.to_string()))).collect();
        List::new(options)
            .block(
                Block::default()
                .borders(Borders::ALL)
                .title(vec![Span::styled(key, Style::default().add_modifier(Modifier::BOLD)),Span::raw(name)])
                .title_alignment(Alignment::Center)
            )
            .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::Black))
            .highlight_symbol("> ")
    }

    let list = create_opt_list(&App::SEARCH_MODE_OPTS, " s", " Search Mode ");
    f.render_stateful_widget(list, rects[0], &mut app.search_mode);

    let list = create_opt_list(&App::DATATYPE_OPTS, " t", " Value Type ");
    f.render_stateful_widget(list, rects[1], &mut app.search_datatype);

    let list = create_opt_list(&App::MATCH_MODE_OPTS, " m", " Match Mode ");
    f.render_stateful_widget(list, rects[2], &mut app.search_type);
    

    // Input Popup
    if matches!(app.edit_state, EditState::Edit) {
        let percent_x = 60;
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Length(3),
                    Constraint::Percentage(40),
                ]
                .as_ref(),
            )
            .split(size);

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1];
        

        let width = area.width.max(3) - 3;
        let scroll = (app.mismem_input.cursor() as u16).max(width) - width;
        let input = Paragraph::new(app.mismem_input.value())
            .style(Style::default().fg(Color::Yellow))
            .scroll((0, scroll))
            .block(Block::default().borders(Borders::ALL).title(format!(" 💉 New Value for {}",app.selected_address)).title_alignment(Alignment::Center));

        f.render_widget(Clear, area);
        f.render_widget(input, area);
        if !app.show_popup {
            f.set_cursor(
                area.x + (app.mismem_input.cursor() as u16).min(width) + 1,
                area.y + 1,
            );
        }
    }


    // Error Popup
    if app.show_popup {
        let area = centered_rect(60, 20, size);
        
        let block = Block::default().title(" Error ").title_alignment(Alignment::Center).borders(Borders::ALL).style(Style::default().fg(Color::Yellow));
        
        let msg = Text::from(app.popup_error.clone());
        let msg = Paragraph::new(msg).alignment(Alignment::Center);
        
        let rects = Layout::default()
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(block.inner(area));
            
        f.render_widget(Clear, area);
        f.render_widget(block, area);
        f.render_widget(msg, rects[1]);
    }
    
}


fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

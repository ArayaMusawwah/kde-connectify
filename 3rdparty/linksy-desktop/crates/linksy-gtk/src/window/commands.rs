//! Saved commands: presets the computer runs when a paired phone asks. Like the KDE
//! Connect run-command plugin, but managed from here and shared over the IPC socket.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;
use linksy_ipc::CommandEntry;
use linksy_i18n::t;

use super::Window;

type Handler = Rc<RefCell<Option<Rc<dyn Fn(CommandAction)>>>>;

pub(super) enum CommandAction {
    Add,
    Edit(CommandEntry),
    Remove(String),
}

pub struct CommandsPage {
    pub root: gtk::ScrolledWindow,
    list: gtk::ListBox,
    empty: gtk::Box,
    handler: Handler,
}

impl CommandsPage {
    pub fn new() -> Rc<Self> {
        let body = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_top(12)
            .margin_bottom(24)
            .margin_start(16)
            .margin_end(16)
            .build();

        let title = gtk::Label::builder()
            .label(t!("Saved Commands"))
            .css_classes(["title-1"])
            .xalign(0.0)
            .build();
        body.append(&title);

        let subtitle = gtk::Label::builder()
            .label(t!("Commands phones can run on this computer. They execute locally, as you."))
            .wrap(true)
            .xalign(0.0)
            .css_classes(["dim-label"])
            .build();
        body.append(&subtitle);

        let add_button = gtk::Button::builder()
            .label(t!("Add Command…"))
            .css_classes(["pill", "suggested-action"])
            .margin_top(12)
            .build();
        body.append(&add_button);

        let empty = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(6)
            .css_classes(["sidebar-empty"])
            .margin_top(24)
            .build();
        empty.append(
            &gtk::Label::builder()
                .label(t!("No Saved Commands"))
                .css_classes(["heading", "dim-label"])
                .build(),
        );
        empty.append(
            &gtk::Label::builder()
                .label(t!("Add one to let paired phones run it on this computer."))
                .wrap(true)
                .justify(gtk::Justification::Center)
                .css_classes(["caption", "dim-label"])
                .build(),
        );
        body.append(&empty);

        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .margin_top(8)
            .build();
        body.append(&list);

        let clamp = adw::Clamp::builder().maximum_size(640).child(&body).build();
        let root = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .child(&clamp)
            .build();

        let handler: Handler = Rc::new(RefCell::new(None));

        let add_handler = handler.clone();
        let add_btn = add_button.clone();
        add_btn.connect_clicked(move |_| {
            if let Some(f) = add_handler.borrow().clone() {
                f(CommandAction::Add);
            }
        });

        Rc::new(Self { root, list, empty, handler })
    }

    pub fn set_handler(&self, f: impl Fn(CommandAction) + 'static) {
        *self.handler.borrow_mut() = Some(Rc::new(f));
    }

    pub fn show(&self, commands: &[CommandEntry]) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        let has = !commands.is_empty();
        self.list.set_visible(has);
        self.empty.set_visible(!has);
        for entry in commands {
            self.list.append(&self.row(entry));
        }
    }

    fn row(&self, entry: &CommandEntry) -> gtk::ListBoxRow {
        let name = gtk::Label::builder()
            .label(glib::markup_escape_text(&entry.name))
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .css_classes(["heading"])
            .build();
        let cmd = gtk::Label::builder()
            .label(glib::markup_escape_text(&entry.command))
            .xalign(0.0)
            .wrap(true)
            .css_classes(["caption", "dim-label"])
            .build();
        let labels = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(3)
            .hexpand(true)
            .valign(gtk::Align::Center)
            .build();
        labels.append(&name);
        labels.append(&cmd);

        let edit_btn = gtk::Button::builder()
            .icon_name("document-edit-symbolic")
            .tooltip_text(t!("Edit"))
            .css_classes(["flat"])
            .valign(gtk::Align::Center)
            .build();
        let (h_edit, entry_clone) = (self.handler.clone(), entry.clone());
        edit_btn.connect_clicked(move |_| {
            if let Some(f) = h_edit.borrow().clone() {
                f(CommandAction::Edit(entry_clone.clone()));
            }
        });

        let remove_btn = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text(t!("Remove"))
            .css_classes(["flat", "destructive-label"])
            .valign(gtk::Align::Center)
            .build();
        let (h_remove, key) = (self.handler.clone(), entry.key.clone());
        remove_btn.connect_clicked(move |_| {
            if let Some(f) = h_remove.borrow().clone() {
                f(CommandAction::Remove(key.clone()));
            }
        });

        let actions = gtk::Box::builder()
            .spacing(4)
            .valign(gtk::Align::Center)
            .build();
        actions.append(&edit_btn);
        actions.append(&remove_btn);

        let row_box = gtk::Box::builder()
            .spacing(12)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(12)
            .margin_end(8)
            .build();
        row_box.append(&labels);
        row_box.append(&actions);

        let row = gtk::ListBoxRow::builder()
            .child(&row_box)
            .activatable(true)
            .build();
        row.set_widget_name(&entry.key);

        // Row activation via the ListBox's row_activated signal (single click on
        // the row, excluding button clicks). The GestureClick was removed because
        // it fired alongside the edit/remove button clicks, causing double dialogs.
        row
    }
}

impl Window {
    pub(super) fn setup_commands(self: &Rc<Self>) {
        let w = Rc::downgrade(self);
        self.commands.set_handler(move |action| {
            if let Some(w) = w.upgrade() {
                w.command_action(action);
            }
        });
    }

    /// Saved commands, managed from here: a page of its own when open.
    pub(super) fn open_commands(self: &Rc<Self>) {
        self.state.borrow_mut().commands_open = true;
        self.paired_list.unselect_all();
        self.render_page();
        self.show_content();
    }

    pub(super) fn refresh_commands(self: &Rc<Self>) {
        let Some(c) = self.client() else { return };
        let w = self.clone();
        let call = self.bridge.call(&c, |c| async move { c.commands_list().await });
        glib::spawn_future_local(async move {
            if let Ok(commands) = call.await {
                w.state.borrow_mut().commands = commands;
                w.render_commands();
            }
        });
    }

    pub(super) fn render_commands(&self) {
        let commands = self.state.borrow().commands.clone();
        self.commands.show(&commands);
    }

    fn command_action(self: &Rc<Self>, action: CommandAction) {
        match action {
            CommandAction::Add => self.add_command_dialog(),
            CommandAction::Edit(entry) => self.edit_command_dialog(&entry),
            CommandAction::Remove(key) => self.remove_command(&key),
        }
    }

    /// Prompt for a name and command, then add it.
    fn add_command_dialog(self: &Rc<Self>) {
        let name = gtk::Entry::builder().placeholder_text(t!("Name")).build();
        let cmd = gtk::Entry::builder().placeholder_text(t!("Command")).build();
        let list = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(8).build();
        list.append(&name);
        list.append(&cmd);
        let dialog = adw::AlertDialog::builder()
            .heading(t!("Add Command"))
            .extra_child(&list)
            .build();
        dialog.add_response("cancel", t!("Cancel"));
        dialog.add_response("add", t!("Add"));
        dialog.set_response_appearance("add", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("add"));
        dialog.set_close_response("cancel");
        dialog.set_response_enabled("add", false);

        let d = dialog.clone();
        let ne = name.clone();
        let ce = cmd.clone();
        name.connect_changed(move |_| {
            d.set_response_enabled("add", !ne.text().trim().is_empty() && !ce.text().trim().is_empty());
        });
        let d = dialog.clone();
        let ne = name.clone();
        let ce = cmd.clone();
        cmd.connect_changed(move |_| {
            d.set_response_enabled("add", !ne.text().trim().is_empty() && !ce.text().trim().is_empty());
        });

        let w = Rc::downgrade(self);
        let name_clone = name.clone();
        let cmd_clone = cmd.clone();
        dialog.connect_response(None, move |_, response| {
            if response != "add" {
                return;
            }
            let n = name_clone.text().trim().to_string();
            let command = cmd_clone.text().trim().to_string();
            let Some(w) = w.upgrade() else { return };
            let Some(client) = w.client() else { return };
            let call = w.bridge.call(&client, move |c| async move { c.commands_add(n.clone(), command.clone()).await });
            let w = Rc::downgrade(&w);
            glib::spawn_future_local(async move {
                let Some(w) = w.upgrade() else { return };
                match call.await {
                    Ok(()) => {
                        w.refresh_commands();
                        w.toasts.add_toast(adw::Toast::new(&t!("Added")));
                    }
                    Err(e) => w.toast_error(&e, None),
                }
            });
        });
        dialog.present(Some(&self.window));
        name.grab_focus();
    }

    /// Prompt to edit an existing saved command.
    fn edit_command_dialog(self: &Rc<Self>, entry: &CommandEntry) {
        let name = gtk::Entry::builder()
            .placeholder_text(t!("Name"))
            .text(&entry.name)
            .build();
        let cmd = gtk::Entry::builder()
            .placeholder_text(t!("Command"))
            .text(&entry.command)
            .build();
        let list = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(8).build();
        list.append(&name);
        list.append(&cmd);
        let dialog = adw::AlertDialog::builder()
            .heading(t!("Edit Command"))
            .extra_child(&list)
            .build();
        dialog.add_response("cancel", t!("Cancel"));
        dialog.add_response("save", t!("Save"));
        dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("save"));
        dialog.set_close_response("cancel");

        let d = dialog.clone();
        let ne = name.clone();
        let ce = cmd.clone();
        let check = move || {
            d.set_response_enabled("save", !ne.text().trim().is_empty() && !ce.text().trim().is_empty());
        };
        let c1 = check.clone();
        name.connect_changed(move |_| c1());
        let c2 = check.clone();
        cmd.connect_changed(move |_| c2());

        let w = Rc::downgrade(self);
        let key = entry.key.clone();
        let name_clone = name.clone();
        let cmd_clone = cmd.clone();
        dialog.connect_response(None, move |_, response| {
            if response != "save" {
                return;
            }
            let n = name_clone.text().trim().to_string();
            let command = cmd_clone.text().trim().to_string();
            let Some(w) = w.upgrade() else { return };
            let Some(client) = w.client() else { return };
            let k = key.clone();
            let call = w.bridge.call(&client, move |c| async move { c.commands_edit(k, n, command).await });
            let w = Rc::downgrade(&w);
            glib::spawn_future_local(async move {
                let Some(w) = w.upgrade() else { return };
                match call.await {
                    Ok(()) => {
                        w.refresh_commands();
                        w.toasts.add_toast(adw::Toast::new(&t!("Saved")));
                    }
                    Err(e) => w.toast_error(&e, None),
                }
            });
        });
        dialog.present(Some(&self.window));
        name.grab_focus();
    }

    /// Remove a saved command, then re-fetch the list.
    fn remove_command(self: &Rc<Self>, key: &str) {
        let Some(c) = self.client() else { return };
        let key = key.to_string();
        let call = self.bridge.call(&c, move |c| async move { c.commands_remove(key.clone()).await });
        let w = self.clone();
        glib::spawn_future_local(async move {
            match call.await {
                Ok(()) => {
                    w.refresh_commands();
                    w.toasts.add_toast(adw::Toast::new(&t!("Removed")));
                }
                Err(e) => w.toast_error(&e, None),
            }
        });
    }
}

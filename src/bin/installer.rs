use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

// Embed resources
const TERMINAL_BINARY: &[u8] = include_bytes!("../../target/release/terminal.exe");
const LOGO_BYTES: &[u8] = include_bytes!("logo.png");

struct InstallerApp {
    step: InstallerStep,
    username: String,
    install_path: String,
    status_message: String,
    progress: f32,
    installation_complete: bool,
}

#[derive(PartialEq)]
enum InstallerStep {
    Welcome,
    UserDetails,
    InstallPath,
    Installing,
    Finished,
}

impl Default for InstallerApp {
    fn default() -> Self {
        Self {
            step: InstallerStep::Welcome,
            username: whoami::username(),
            install_path: get_default_install_path(),
            status_message: String::new(),
            progress: 0.0,
            installation_complete: false,
        }
    }
}

fn get_default_install_path() -> String {
    if let Some(mut path) = dirs::data_local_dir() {
        path.push("NebulaCLI");
        return path.to_string_lossy().to_string();
    }
    r"C:\NebulaCLI".to_string()
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut next_step = None;

        // Custom Panel for a "Premium" look
        egui::CentralPanel::default().show(ctx, |ui| {
            // Main Layout: Side Logo + Content or Top Logo + Content
            // Let's go with a Top Logo centered for a modern wizard look

            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                // Logo
                ui.add(
                    egui::Image::new(egui::include_image!("logo.png"))
                        .max_width(120.0)
                        .corner_radius(10.0),
                );
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("NebulaCLI").size(24.0).strong());
                ui.label(
                    egui::RichText::new("The Future of Command Line")
                        .italics()
                        .weak(),
                );
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            // Content Area
            let content_rect = ui.available_rect_before_wrap();
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                ui.vertical_centered(|ui| {
                    match self.step {
                        InstallerStep::Welcome => {
                            ui.heading("Welcome");
                            ui.add_space(10.0);
                            ui.label(
                                "This wizard will guide you through the installation of NebulaCLI.",
                            );
                            ui.label("Get ready for a superior terminal experience.");
                            ui.add_space(30.0);

                            style_button(ui, "Get Started", |ui| {
                                if ui
                                    .button(egui::RichText::new("Get Started").size(16.0))
                                    .clicked()
                                {
                                    next_step = Some(InstallerStep::UserDetails);
                                }
                            });
                        }
                        InstallerStep::UserDetails => {
                            ui.heading("User Setup");
                            ui.add_space(10.0);
                            ui.label("How should we call you?");
                            ui.add_space(5.0);

                            ui.add(
                                egui::TextEdit::singleline(&mut self.username)
                                    .hint_text("Enter your username")
                                    .min_size(egui::vec2(200.0, 30.0)),
                            );

                            ui.add_space(30.0);

                            ui.horizontal(|ui| {
                                ui.add_space((ui.available_width() - 200.0) / 2.0); // Center grouping
                                if ui.button("Back").clicked() {
                                    next_step = Some(InstallerStep::Welcome);
                                }
                                if ui.button(egui::RichText::new("Next").strong()).clicked() {
                                    if !self.username.trim().is_empty() {
                                        next_step = Some(InstallerStep::InstallPath);
                                    } else {
                                        self.status_message =
                                            "Username cannot be empty!".to_string();
                                    }
                                }
                            });
                        }
                        InstallerStep::InstallPath => {
                            ui.heading("Installation Path");
                            ui.add_space(10.0);
                            ui.label("Where should NebulaCLI be installed?");

                            ui.horizontal(|ui| {
                                ui.label("Path:");
                                ui.text_edit_singleline(&mut self.install_path);
                                if ui.button("📂").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.install_path = path.to_string_lossy().to_string();
                                    }
                                }
                            });

                            ui.add_space(30.0);

                            ui.horizontal(|ui| {
                                ui.add_space((ui.available_width() - 200.0) / 2.0);
                                if ui.button("Back").clicked() {
                                    next_step = Some(InstallerStep::UserDetails);
                                }
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("Install Now")
                                                .strong()
                                                .color(egui::Color32::WHITE),
                                        )
                                        .fill(egui::Color32::from_rgb(0, 120, 215)),
                                    ) // Accent color
                                    .clicked()
                                {
                                    next_step = Some(InstallerStep::Installing);
                                }
                            });
                        }
                        InstallerStep::Installing => {
                            ui.heading("Installing NebulaCLI...");
                            ui.add_space(20.0);

                            let progress_bar = egui::ProgressBar::new(self.progress)
                                .show_percentage()
                                .animate(true);
                            ui.add(progress_bar);

                            ui.add_space(10.0);
                            ui.label(if self.installation_complete {
                                "Installation Finished!"
                            } else {
                                "Copying files and configuring system..."
                            });

                            if !self.installation_complete {
                                self.progress += 0.02; // Slower progress for effect
                                if self.progress >= 1.0 {
                                    if let Err(e) =
                                        install_application(&self.install_path, &self.username)
                                    {
                                        self.status_message = format!("Error: {}", e);
                                    } else {
                                        self.installation_complete = true;
                                        next_step = Some(InstallerStep::Finished);
                                    }
                                }
                                ctx.request_repaint();
                                thread::sleep(Duration::from_millis(30));
                            }
                        }
                        InstallerStep::Finished => {
                            ui.heading("All Set!");
                            ui.add_space(10.0);
                            ui.label("NebulaCLI has been successfully installed.");
                            ui.label(format!("Location: {}", self.install_path));
                            ui.add_space(30.0);

                            if ui
                                .button(egui::RichText::new("Launch & Close").size(16.0))
                                .clicked()
                            {
                                // Optional: Launch the app
                                std::process::exit(0);
                            }
                        }
                    }

                    if !self.status_message.is_empty() {
                        ui.add_space(10.0);
                        ui.colored_label(egui::Color32::RED, &self.status_message);
                    }
                });
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("v1.0.0").weak().size(10.0));
            });
        });

        if let Some(step) = next_step {
            self.step = step;
            self.status_message.clear();
        }
    }
}

fn style_button(ui: &mut egui::Ui, _text: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.scope(|ui| {
        // ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(45, 45, 45);
        add_contents(ui);
    });
}

fn add_to_start_menu(exe_path: &Path, install_dir: &Path) {
    if let Some(mut start_menu) = dirs::data_local_dir() {
        start_menu.push("Microsoft");
        start_menu.push("Windows");
        start_menu.push("Start Menu");
        start_menu.push("Programs");

        if start_menu.exists() {
            let link_path = start_menu.join("NebulaCLI.lnk");
            let script = format!(
                "$s=(New-Object -COM WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.WorkingDirectory='{}';$s.Save()",
                link_path.to_string_lossy(),
                exe_path.to_string_lossy(),
                install_dir.to_string_lossy()
            );
            std::process::Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(&script)
                .output()
                .ok();
        }
    }
}

fn add_to_registry_context_menu(exe_path: &Path) -> std::io::Result<()> {
    use winreg::RegKey;
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // 1. Directory Context Menu (Background)
    let path = Path::new("Software")
        .join("Classes")
        .join("Directory")
        .join("Background")
        .join("shell")
        .join("NebulaCLI");
    let (key, _) = hkcu.create_subkey(&path)?;
    key.set_value("", &"Open NebulaCLI here")?;
    key.set_value("Icon", &exe_path.to_string_lossy().as_ref())?;

    let (cmd_key, _) = key.create_subkey("command")?;
    cmd_key.set_value("", &format!("\"{}\"", exe_path.to_string_lossy()))?;

    // 2. Directory Context Menu (Folder)
    let path_dir = Path::new("Software")
        .join("Classes")
        .join("Directory")
        .join("shell")
        .join("NebulaCLI");
    let (key_dir, _) = hkcu.create_subkey(&path_dir)?;
    key_dir.set_value("", &"Open NebulaCLI here")?;
    key_dir.set_value("Icon", &exe_path.to_string_lossy().as_ref())?;

    let (cmd_key_dir, _) = key_dir.create_subkey("command")?;
    cmd_key_dir.set_value("", &format!("\"{}\"", exe_path.to_string_lossy()))?;

    Ok(())
}

fn add_to_user_path(install_dir: &Path) -> std::io::Result<()> {
    use winreg::RegKey;
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    let current_path: String = env_key.get_value("Path")?;
    let new_entry = install_dir.to_string_lossy();

    if !current_path.contains(new_entry.as_ref()) {
        let new_path = if current_path.ends_with(';') {
            format!("{}{}", current_path, new_entry)
        } else {
            format!("{};{}", current_path, new_entry)
        };
        env_key.set_value("Path", &new_path)?;
    }

    Ok(())
}

fn install_application(path_str: &str, username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let install_dir = Path::new(path_str);
    if !install_dir.exists() {
        fs::create_dir_all(install_dir)?;
    }

    // 1. Write Executable
    let exe_path = install_dir.join("NebulaCLI.exe");
    fs::write(&exe_path, TERMINAL_BINARY)?;

    // 2. Write Config (Plain text as expected by config.rs)
    let config_path = install_dir.join("terminal_config.txt");
    fs::write(&config_path, username)?;

    // 3. Create Desktop Shortcut (Simple PowerShell invocation)
    if let Some(desktop) = dirs::desktop_dir() {
        let link_path = desktop.join("NebulaCLI.lnk");
        let script = format!(
            "$s=(New-Object -COM WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.WorkingDirectory='{}';$s.Save()",
            link_path.to_string_lossy(),
            exe_path.to_string_lossy(),
            install_dir.to_string_lossy()
        );
        std::process::Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(&script)
            .output()?;
    }

    // 4. Create Start Menu Shortcut
    add_to_start_menu(&exe_path, install_dir);

    // 5. Add to Context Menu
    add_to_registry_context_menu(&exe_path).ok();

    // 6. Add to PATH
    add_to_user_path(install_dir).ok();

    Ok(())
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0]) // Slightly larger for better layout
            .with_resizable(false)
            .with_icon(eframe::icon_data::from_png_bytes(LOGO_BYTES).unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "NebulaCLI Setup",
        options,
        Box::new(|cc| {
            // Install image loaders
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // Setup styles
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals = egui::Visuals::dark(); // Dark theme for premium feel
            // Increase text size slightly
            for text_style in [
                egui::TextStyle::Body,
                egui::TextStyle::Button,
                egui::TextStyle::Heading,
            ] {
                // style.text_styles.get_mut(&text_style).unwrap().size += 2.0;
            }
            cc.egui_ctx.set_style(style);

            Ok(Box::new(InstallerApp::default()))
        }),
    )
}

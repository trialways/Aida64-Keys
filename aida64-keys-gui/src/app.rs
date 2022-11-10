use chrono::{Date, Duration, TimeZone, Utc};
use clipboard::{ClipboardContext, ClipboardProvider};
use eframe::{egui, epi};
use egui_datepicker::DatePicker;
use std::ops::{Add, Sub};
use strum::IntoEnumIterator;

use aida64_keys_lib::{KeyEdition, License};

pub struct App {
    keys: Vec<String>,
    key_amount: i32,
    key_seats: i32,
    key_edition: KeyEdition,
    key_purchase: Date<Utc>,
    key_expire: Date<Utc>,
    key_expire_never: bool,
    key_maintenance_expire: Date<Utc>,

    clipboard: Option<ClipboardContext>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            key_seats: 1,
            key_amount: 1,
            key_edition: KeyEdition::Extreme,
            key_purchase: Utc::today(),
            key_expire: Utc::today().add(Duration::days(1)),
            key_expire_never: true,
            key_maintenance_expire: Utc::today() + Duration::days(3658),

            clipboard: ClipboardProvider::new().ok(),
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Aida64 Key Generator"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].group(|ui| {
                    let available_size = ui.available_size();

                    ui.set_max_size(available_size);
                    ui.set_min_size(available_size);

                    ui.horizontal(|ui| {
                        if ui.button("Generate").clicked() {
                            self.keys.clear();

                            (0..self.key_amount).for_each(|_| {
                                let mut license = License::new(self.key_edition)
                                    .with_seats(self.key_seats)
                                    .with_purchase_date(self.key_purchase)
                                    .with_maintenance_expiry(Duration::days(
                                        self.key_maintenance_expire
                                            .sub(self.key_purchase)
                                            .num_days(),
                                    ));

                                if !self.key_expire_never {
                                    println!("{:?}", self.key_expire);

                                    license = license.with_license_expiry(Some(self.key_expire));
                                }

                                // TODO!: currently not dedupping
                                self.keys.push(license.generate_string(true));
                            });
                        }

                        ui.add(egui::DragValue::new(&mut self.key_amount).clamp_range(1..=1000));
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut self.key_seats).clamp_range(1..=797));
                        ui.label("Seats");
                    });

                    egui::ComboBox::new("edition_combobox", "Edition")
                        .selected_text(self.key_edition.to_string())
                        .show_ui(ui, |ui| {
                            for edition in KeyEdition::iter() {
                                ui.selectable_value(
                                    &mut self.key_edition,
                                    edition,
                                    edition.to_string(),
                                );
                            }
                        });

                    ui.horizontal(|ui| {
                        ui.add(
                            DatePicker::new("license_purchase_date", &mut self.key_purchase)
                                .min_date(Utc.ymd(2004, 1, 1))
                                .max_date(Utc.ymd(2099, 12, 31)),
                        );
                        ui.label("Purchase Date");
                    });

                    let min_date = self.key_purchase + Duration::days(1);
                    let max_date = self.key_purchase + Duration::days(3658);

                    self.key_expire = self.key_expire.clamp(min_date, max_date);
                    self.key_maintenance_expire =
                        self.key_maintenance_expire.clamp(min_date, max_date);

                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(!self.key_expire_never, |ui| {
                            ui.add(
                                DatePicker::new("license_expire_date", &mut self.key_expire)
                                    .min_date(min_date)
                                    .max_date(max_date),
                            );
                        });

                        ui.label("Expire Date");
                        ui.checkbox(&mut self.key_expire_never, "No Expiry");
                    });

                    ui.horizontal(|ui| {
                        ui.add(
                            DatePicker::new(
                                "maintenance_expire_date",
                                &mut self.key_maintenance_expire,
                            )
                            .min_date(min_date)
                            .max_date(max_date),
                        );
                        ui.label("Maintenance Expire Date");
                    });
                });

                columns[1].group(|ui| {
                    let available_size = ui.available_size();

                    ui.set_max_size(available_size);
                    ui.set_min_size(available_size);

                    egui::ScrollArea::new([false, true]).show(ui, |ui| {
                        for key in &self.keys {
                            if ui
                                .selectable_label(
                                    false,
                                    egui::RichText::new(key).text_style(egui::TextStyle::Monospace),
                                )
                                .clicked()
                            {
                                if let Some(clipboard) = &mut self.clipboard {
                                    let _ = clipboard.set_contents(key.to_string());
                                }
                            }
                        }
                    });
                });
            });
        });
    }
}

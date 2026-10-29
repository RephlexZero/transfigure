mod converter;
mod header;
mod icons;
mod info;
mod queue;
mod specimen;

use leptos::prelude::*;

use crate::state::Batch;
use converter::ConverterSection;
use header::Header;
use info::{Faq, Footer, FormatGrid, Formats, HowItWorks, SupportBanner};

#[component]
pub fn App() -> impl IntoView {
    provide_context(Batch::new());

    view! {
        <div class="min-h-screen flex flex-col overflow-x-clip animate-fade-in">
            <div class="wrap flex-1">
                <Header/>
                <main>
                    <ConverterSection/>
                    <FormatGrid/>
                    <HowItWorks/>
                    <Formats/>
                    <Faq/>
                    <SupportBanner/>
                </main>
                <Footer/>
            </div>
        </div>
    }
}

mod converter;
mod header;
mod hero;
mod info;

use leptos::prelude::*;

use crate::types::BatchFile;
use converter::ConverterSection;
use header::Header;
use hero::Hero;
use info::{Footer, Formats, HowItWorks, PrivacyBanner, SupportBanner};

#[component]
pub fn App() -> impl IntoView {
    let files = RwSignal::new(Vec::<BatchFile>::new());
    let next_id = RwSignal::new(0usize);

    view! {
        <div class="min-h-screen bg-base-300 relative overflow-hidden">
            <div class="pointer-events-none fixed inset-0 z-0">
                <div class="orb orb-1"></div>
                <div class="orb orb-2"></div>
                <div class="orb orb-3"></div>
                <div class="noise"></div>
            </div>

            <div class="relative z-10">
                <Header/>

                <main class="container mx-auto px-4 sm:px-6 lg:px-8 pb-20">
                    <Hero/>
                    <ConverterSection files=files next_id=next_id/>
                    <HowItWorks/>
                    <Formats/>
                    <PrivacyBanner/>
                    <SupportBanner/>
                </main>

                <Footer/>
            </div>
        </div>
    }
}

mod converter;
mod header;
mod hero;
mod icons;
mod info;

use leptos::prelude::*;

use crate::types::BatchFile;
use converter::ConverterSection;
use header::Header;
use hero::{Hero, TrustRow};
use info::{Faq, Footer, Formats, HowItWorks, SupportBanner};

#[component]
pub fn App() -> impl IntoView {
    let files = RwSignal::new(Vec::<BatchFile>::new());
    let next_id = RwSignal::new(0usize);

    view! {
        <div class="relative min-h-screen flex flex-col overflow-x-clip">
            // Soft warm light behind the converter; purely decorative.
            <div
                class="pointer-events-none absolute inset-x-0 top-0 h-[40rem] -z-0 bg-[radial-gradient(60%_50%_at_50%_0%,rgb(var(--accent)/0.10),transparent_70%)]"
                aria-hidden="true"
            ></div>

            <Header/>

            <main class="relative flex-1">
                <section class="px-4 sm:px-6 pt-10 sm:pt-16 pb-16 sm:pb-24">
                    <Hero/>
                    <div class="mx-auto max-w-3xl mt-8 sm:mt-12">
                        <ConverterSection files=files next_id=next_id/>
                    </div>
                    <TrustRow/>
                </section>

                <HowItWorks/>
                <Formats/>
                <Faq/>
                <SupportBanner/>
            </main>

            <Footer/>
        </div>
    }
}

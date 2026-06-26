// @ts-check
import {defineConfig} from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLlmsTxt from 'starlight-llms-txt';

// https://astro.build/config
export default defineConfig({
    site: "https://davidsteiner.github.io",
    base: "/orator",
    integrations: [
        starlight({
            title: 'Orator',
            plugins: [starlightLlmsTxt({projectName: 'Orator'})],
            customCss: ['./src/styles/custom.css'],
            social: [{icon: 'github', label: 'GitHub', href: 'https://github.com/davidsteiner/orator'}],
            sidebar: [
                {
                    label: 'Guides',
                    items: [
                        {label: 'Getting started', slug: 'guides/getting-started'},
                    ],
                },
                {
                    label: 'Reference',
                    autogenerate: {directory: 'reference'},
                },
            ],
        }),
    ],
});

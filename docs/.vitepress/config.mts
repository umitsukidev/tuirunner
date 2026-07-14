import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
    title: "tuirunner",
    description: "A concurrent task runner with a TUI interface built on Ratatui.",
    transformPageData: (pageData, ctx) => {
        const canonicalUrl = `https://tuir.umitsuki.dev/${pageData.relativePath}`
            .replace(/index\.md$/, "")
            .replace(/\.md$/, ".html");
        pageData.frontmatter.head ??= [];
        pageData.frontmatter.head.push(
            [
                "meta",
                {
                    name: "og:title",
                    content:
                        pageData.frontmatter.layout === "home"
                            ? ctx.siteConfig.site.title
                            : `${pageData.title} | ${ctx.siteConfig.site.title}`,
                },
            ],
            [
                "link",
                {
                    rel: "canonical",
                    href: canonicalUrl,
                },
            ],
        );
    },

    locales: {
        root: {
            label: "English",
            lang: "en",
            themeConfig: {
                nav: [
                    { text: "Home", link: "/" },
                    { text: "Guide", link: "/guide" },
                ],
                sidebar: [
                    {
                        text: "Guide",
                        items: [{ text: "User Guide", link: "/guide" }],
                    },
                ],
            },
        },
        ja: {
            label: "日本語",
            lang: "ja",
            link: "/ja/",
            themeConfig: {
                nav: [
                    { text: "ホーム", link: "/ja/" },
                    { text: "ガイド", link: "/ja/guide" },
                ],
                sidebar: [
                    {
                        text: "ガイド",
                        items: [{ text: "ユーザーガイド", link: "/ja/guide" }],
                    },
                ],
            },
        },
    },

    themeConfig: {
        socialLinks: [
            { icon: "github", link: "https://github.com/umitsukidev/tuirunner" },
            {
                icon: {
                    svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-library-icon lucide-library"><path d="m16 6 4 14"/><path d="M12 6v14"/><path d="M8 8v12"/><path d="M4 4v16"/></svg>`,
                },
                link: "https://crates.io/crates/tuirunner",
                ariaLabel: "crates.io",
            },
        ],
        footer: {
            message: "Released under the MIT License.",
            copyright: "Copyright © 2026 kurage (@umitsukidev)",
        },
        externalLinkIcon: true,
    },
});

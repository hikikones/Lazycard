import { IDatabase } from './Database';

const demo = (): IDatabase => {
    const now = new Date(Date.now()).toISOString();

    return {
        version: "1.1.0",
        topics: [{id: 1, name: "Demo"}],
        cards: [
            {
                id: 1,
                front: "Format cards with **Markdown**.",
                back: "* _italic_\n* ~~strikethrough~~\n* `inline code`",
                dueDate: now,
                dueDays: 0,
                attempts: 0,
                successes: 0,
                topicId: 1
            },
            {
                id: 2,
                front: "Write mathematics with $\\KaTeX$.",
                back: "$$ \\int_a^b x^2 \\ \\mathrm dx $$",
                dueDate: now,
                dueDays: 0,
                attempts: 0,
                successes: 0,
                topicId: 1
            },
            {
                id: 3,
                front: "Render code blocks with **syntax highlighting**.",
                back: "```python\ndef add(a, b):\n    return a + b\n```",
                dueDate: now,
                dueDays: 0,
                attempts: 0,
                successes: 0,
                topicId: 1
            },
            {
                id: 4,
                front: "Add **tables** to your cards.",
                back: "Tables| Are | Cool\n:--- | :---: | ---:\nleft | center | right\n1 | 2 | 3",
                dueDate: now,
                dueDays: 0,
                attempts: 0,
                successes: 0,
                topicId: 1
            },
            {
                id: 5,
                front: "Display **images** by copy and pasting.",
                back: "![](data:image/png;base64,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)\n\nBut be careful, they are encoded in `base64` so performance drops considerably if they are too big. Stick with small images.",
                dueDate: now,
                dueDays: 0,
                attempts: 0,
                successes: 0,
                topicId: 1
            },
            {
                id: 6,
                front: "Use **keyboard shortcuts** for quicker reviewing.",
                back: "When reviewing, hit `Space` for showing answer, `ArrowUp` for successful recall, `ArrowDown` for unsuccessful, and `ArrowRight` for skipping a card.",
                dueDate: now,
                dueDays: 0,
                attempts: 0,
                successes: 0,
                topicId: 1
            },
            {
                id: 7,
                front: "Retain more information with **spaced repetition**.",
                back: "For each card you recall correctly, the amount of days until next review are _increased_. If not, the days are _decreased_, with a minimum of _one_ day.",
                dueDate: now,
                dueDays: 0,
                attempts: 0,
                successes: 0,
                topicId: 1
            },
            {
                id: 8,
                front: "Open [__links__](https://en.wikipedia.org/wiki/Spaced_repetition) in a new window.",
                back: "But what will it open though.",
                dueDate: now,
                dueDays: 0,
                attempts: 0,
                successes: 0,
                topicId: 1
            }
        ]
    }
}

export default demo;
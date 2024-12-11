<div align="center">

# ✒️ lazycard

A simple flashcard application for the terminal.

TODO: image

</div>

## 📌 Features

- Write cards in a light markup
- Card scheduling using [FSRS](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm)
- Readable database in [RON](https://github.com/ron-rs/ron)

## ⚡ Usage

The `lazycard` command takes one mandatory argument, which is the path to your database file.

```
lazycard /path/to/my/database.ron
```

In addition it comes with two optional arguments.

| Option | Description |
| ------ | ----------- |
| `--desired-retention <PERCENT>` | The desired retention in percent for your cards. Default value is 80. See [optimal retention](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Optimal-Retention) for more information. |
| `--external-editor` | Write cards in your default text editor. |

## 📜 Markup

Cards are written in a custom lightweight markup language, inspired by both [Markdown](https://en.wikipedia.org/wiki/Markdown) and [Djot](https://djot.net/). It provides a small set of syntax for formatting text in the terminal. The following table shows the entire syntax available.

<table align="center">
<tr>
<th>Markup</th>
<th>Result</th>
</tr>
<tr>
<td>

<pre>
A normal paragraph with *bold* and _italic_ text.

| Center paragraph

# This is a comment
> Right paragraph

---

- item 1
- item 2

```python
def add(a, b):
    return a + b
```
</pre>

</td>
<td>

A normal paragraph with **bold** and _italic_ text.

<p align="center">Center paragraph</p>

<p align="right">Right paragraph</p>

<hr>

- item 1
- item 2

<pre>
def add(a, b):
    return a + b
</pre>

</td>
</tr>
</table>

## 🔖 Install

todo

## 🔧 Development

todo
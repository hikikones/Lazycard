import { Topic, Card } from '../model/Database';
import md from '../controller/Markdown';

const html = (topic: Topic, cards: Card[]) => {
    return `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Lazycard - ${topic.name}</title>

    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.css">
    <script defer src="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.js"></script>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/themes/prism.min.css">

    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/prism.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-python.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-java.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-javascript.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-typescript.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-csharp.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-css.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-c.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-cpp.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-swift.min.js"></script>

    <link rel="stylesheet" href="https://unpkg.com/spectre.css/dist/spectre.min.css">

    <script>
        function save() {
            var data = {
                "name": "${topic.name}",
                "cards": [ ${cards.map(c => `{
                    "front": \`${c.front.replace(/\\/g,"\\\\").replace(/`/g,"\\`")}\`,
                    "back": \`${c.back.replace(/\\/g,"\\\\").replace(/`/g,"\\`")}\`
                },`).join('')} ]
            };
            var json = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(data, null, 2));
            var link = document.createElement("a");
            link.setAttribute("href", json);
            link.setAttribute("download", "${topic.name}.lazytopic");
            link.click();
        }
    </script>

    <style>
        body {margin: 2rem auto; max-width: 1360px}
        h1.topic {text-align: center}
        button {margin: 1rem 0}
        .cards {display:grid; grid-gap: 1rem; grid-template-columns: repeat(auto-fill, 400px); justify-content: center}
        .card {padding: 1rem; box-shadow: 0 0.25rem 1rem rgba(9, 9, 10, 0.15)}
        hr {width: 100%; border: none; border-top: 1px solid lightgray; margin: 0; margin-bottom: 1rem}
        img {display: block; margin: 0 auto; max-width: 100%; height: auto}
        table {width: 100%; border-spacing: 0; border: 1px solid whitesmoke}
        table thead {background-color: whitesmoke}
        thead th, tbody td {padding: 0.25rem; border: 1px solid whitesmoke}
        ul, ol {margin-top: 0}
    </style>
</head>
<body>
    <h1 class="topic">${topic.name}</h1>
    <button class="btn btn-primary p-centered" onClick="save()">Save</button>
    <div class="cards">
        ${cards.map(c =>`<div class="card">${md.parse(c.front)}<hr/>${md.parse(c.back)}</div>`).join('')}
    </div>
</body>
</html>
    `
}

export default html;
import * as React from 'react';
import * as ReactDOM from 'react-dom';

import db from './model/Database';
 
const Index = () => {
    return <div>Hello, World!</div>;
};
 
ReactDOM.render(<Index />, document.getElementById('app'));




// test
// const topic = db.topics.new("Math");
// topic.id = 1;
// const card = db.cards.new(topic.id);
// card.front = "HELLO";
// console.log(card);

// db.topics.add(topic);
// db.cards.add(card);



//db.load();
//db.save();

const test = db.cards.new(1);
db.cards.add(test);
test.front = "NEW FRONT LOLOLOL okkkk";
console.log(db.cards);
console.log(db.topics);
import * as React from "react";

import md from "../controller/Markdown";

const Card = (props: ICardViewProps) => {
  const back = () => {
    if (!props.showBack || props.back === "") return null;

    return (
      <>
        <hr />
        <CardContent markdown={props.back} />
      </>
    );
  };

  return (
    <div id="card" className="card shadow">
      <CardContent markdown={props.front} />
      {back()}

      {props.children || null}
    </div>
  );
};

interface ICardViewProps {
  front: string;
  back: string;
  showBack: boolean;
  children?: React.ReactNode;
}

const CardContent = (props: { markdown: string }) => {
  return <div dangerouslySetInnerHTML={{ __html: md.parse(props.markdown) }} />;
};

export default Card;

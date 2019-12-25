import * as React from 'react';

import db, { Card } from '../model/Database';

import CardSelectable from './CardSelectable';
import Button from './Button';
import Dropdown, { DropdownItem } from './Dropdown';
import SearchInput from './SearchInput';
import Modal from './Modal';

export default class Cards extends React.Component<IProps, IState> {
    private readonly search = React.createRef<SearchInput>();
    private readonly topics = React.createRef<HTMLSelectElement>();
    private cards: Card[];

    public constructor(props: IProps) {
        super(props);
        this.state = {
            showBack: false,
            selected: this.props.cards.filter(c => c.selected).length,
            showModal: false,
        }
    }

    private toggleAnswer = (): void => {
        this.setState({ showBack: !this.state.showBack });
    }

    private select = (): void => {
        this.setState({ selected: this.state.selected + 1 });
        this.props.onCardChange();
    }

    private deselect = (): void => {
        this.setState({ selected: this.state.selected - 1 });
        this.props.onCardChange();
    }

    private toggleSelectAll = (): void => {
        if (this.isAllSelected()) {
            (this.cards || this.props.cards).forEach(c => c.selected = false);
        } else {
            (this.cards || this.props.cards).forEach(c => c.selected = true);
        }
        this.setState({ selected: this.props.cards.filter(c => c.selected).length });
        this.props.onCardChange();
    }

    private isAllSelected = (): boolean => {
        return this.state.selected === this.props.cards.length;
    }

    private delete = (card: Card): void => {
        if (card.selected) this.deselect();
    }

    private onSearch = (): void => {
        if (this.search.current.isEmpty()) {
            this.cards = null;
        } else {
            this.cards = this.search.current.query([...this.props.cards]);
        }
        this.forceUpdate()
    }

    private toggleModal = (): void => {
        this.setState({ showModal: !this.state.showModal });
    }

    private move = (): void => {
        const selectedCards = this.props.cards.filter(c => c.selected);
        if (selectedCards.length === 0) return;
        selectedCards.forEach(c => {
            c.topicId = Number(this.topics.current.value);
            c.selected = false;
        });
        this.props.onCardChange();
        this.toggleModal();
    }

    public render() {
        if (this.props.cards.length === 0) return null;

        let cards = [...this.props.cards];
        cards.sort((a, b) => b.id - a.id);

        return (
            <div>
                <h2>Cards</h2>

                <section>
                    <Button
                        name="Show answer"
                        icon={this.state.showBack ? "check_box" : "check_box_outline_blank"}
                        action={this.toggleAnswer}
                    />
                    <Button
                        name="Select all"
                        icon={this.isAllSelected() ? "check_box" : "check_box_outline_blank"}
                        action={this.toggleSelectAll}
                    />
                    <Dropdown name="Bulk" icon="assignment" number={this.state.selected} showDownArrow={true}>
                        <DropdownItem name="Move" icon="arrow_forward" action={this.toggleModal} />
                    </Dropdown>
                    <SearchInput ref={this.search} onInput={this.onSearch} />
                </section>

                <section className="cards">
                    {(this.cards || cards).map(c =>
                        <CardSelectable
                            key={c.id}
                            card={c}
                            showBack={this.state.showBack}
                            onDelete={this.delete}
                            onSelect={this.select}
                            onDeselect={this.deselect}
                        />
                    )}
                </section>

                <Modal show={this.state.showModal} onClickOutside={this.toggleModal}>
                    <h2>Move</h2>
                    <p>Move selected cards to another topic.</p>
                    <label>Topics</label>
                    <select ref={this.topics}>
                        {db.topics.getAll().map(t => <option key={t.id} value={t.id}>{t.name}</option>)}
                    </select>
                    <Button name="Move" icon="done" action={this.move} />
                    <Button name="Cancel" icon="close" action={this.toggleModal} />
                </Modal>
            </div>
        );
    }
}

interface IProps {
    cards: readonly Card[]
    onCardChange(): void
}

interface IState {
    showBack: boolean
    selected: number
    showModal: boolean
}
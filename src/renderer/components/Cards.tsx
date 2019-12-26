import * as React from 'react';

import db, { Card } from '../model/Database';
import search from '../controller/Search';

import CardSelectable from './CardSelectable';
import Button from './Button';
import Dropdown, { DropdownItem } from './Dropdown';
import Modal from './Modal';

enum CardSort {
    Newest,
    Oldest,
    RetentionRateAsc,
    RetentionRateDesc
}

export default class Cards extends React.Component<IProps, IState> {
    private readonly searchInput = React.createRef<HTMLInputElement>();
    private readonly topics = React.createRef<HTMLSelectElement>();
    private cards: Card[];

    public constructor(props: IProps) {
        super(props);
        this.state = {
            showBack: false,
            selected: this.props.cards.filter(c => c.selected).length,
            showModal: false,
            query: "",
            sort: CardSort.Newest,
            amount: 40
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
            this.cards.forEach(c => c.selected = false);
        } else {
            this.cards.forEach(c => c.selected = true);
        }
        this.setState({ selected: this.props.cards.filter(c => c.selected).length });
        this.props.onCardChange();
    }

    private isAllSelected = (): boolean => {
        return this.state.selected === this.props.cards.length;
    }

    private delete = (card: Card): void => {
        if (card.selected) this.deselect();
        else this.props.onCardChange();
    }

    private onSearch = (): void => {
        this.setState({ query: this.searchInput.current.value });
    }

    private clearSearch = (): void => {
        this.searchInput.current.value = "";
        this.setState({ query: "" });
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
        this.setState({ selected: 0 });
        this.props.onCardChange();
        this.toggleModal();
    }

    private setSort(sortType: CardSort): void {
        this.setState({ sort: sortType });
    }

    private sort = (): void => {
        switch (this.state.sort) {
            case CardSort.Newest:
                this.cards.sort((a, b) => b.id - a.id); break;
            case CardSort.Oldest:
                this.cards.sort((a, b) => a.id - b.id); break;
            case CardSort.RetentionRateDesc:
                this.cards.sort((a, b) => b.retentionRate() - a.retentionRate()); break;
            case CardSort.RetentionRateAsc:
                this.cards.sort((a, b) => a.retentionRate() - b.retentionRate()); break;
            default:
                this.cards.sort((a, b) => b.id - a.id); break;
        }
    }

    private loadMore = () => {
        this.setState({ amount: this.state.amount + 40 });
    }

    private loadAll = () => {
        this.setState({ amount: this.cards.length });
    }

    public render() {
        if (this.props.cards.length === 0) return null;

        this.cards = [...this.props.cards];
        if (this.state.query !== "") this.cards = search.query(this.state.query, this.cards);
        this.sort();

        return (
            <div>
                <h2>Cards</h2>

                <section>
                    <div className="search-container">
                        <i className="search-icon material-icons">search</i>
                        <input
                            ref={this.searchInput}
                            className="search"
                            placeholder="Search..."
                            type="text"
                            onInput={this.onSearch}
                        />
                        <i onClick={this.clearSearch} className="clear-icon material-icons">close</i>
                    </div>
                </section>

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
                    <Dropdown name="Sort" icon="sort" showDownArrow={true}>
                        <DropdownItem name="Newest" icon="arrow_upward" action={() => this.setSort(CardSort.Newest)} />
                        <DropdownItem name="Oldest" icon="arrow_downward" action={() => this.setSort(CardSort.Oldest)} />
                        <DropdownItem name="Retention Rate" icon="trending_down" action={() => this.setSort(CardSort.RetentionRateDesc)} />
                        <DropdownItem name="Retention Rate" icon="trending_up" action={() => this.setSort(CardSort.RetentionRateAsc)} />
                    </Dropdown>
                    <Dropdown name="Bulk" icon="assignment" number={this.state.selected} showDownArrow={true}>
                        <DropdownItem name="Move" icon="arrow_forward" action={this.toggleModal} />
                    </Dropdown>
                </section>

                <section className="cards">
                    {this.cards.slice(0, this.state.amount).map(c =>
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

                {this.state.amount < this.cards.length
                    ?   <section className="row row-center">
                            <Button name="Load More" icon="cached" action={this.loadMore} />
                            <Button name="Load all" icon="done_all" action={this.loadAll} />
                        </section>
                    :   null
                }

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
    query: string
    sort: CardSort
    amount: number
}
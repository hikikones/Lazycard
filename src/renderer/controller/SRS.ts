import { Card } from '../model/Database';

// TODO: Use fibonacci sequence instead

class SRS {
    public schedule(card: Card, success: boolean): void {
        let newDueDays = success ? card.dueDays * 2 : card.dueDays / 2;
        if (newDueDays < 1) newDueDays = 1;
        card.dueDays = newDueDays;
        card.dueDate.setDate(new Date(Date.now()).getDate() + newDueDays);
    }

    public tomorrow(card: Card): void {
        card.dueDays = 1;
        card.dueDate = new Date(Date.now());
        card.dueDate.setDate(card.dueDate.getDate() + card.dueDays);
    }

    public today(card: Card): void {
        card.dueDays = 0;
        card.dueDate = new Date(Date.now());
    }
}

export default new SRS();
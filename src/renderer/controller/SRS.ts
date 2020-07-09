import { Card } from '../model/Database';

class SRS {
    public schedule(card: Card, success: boolean): void {
        const newDueDays = this.fib(card.dueDays, success);
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

    private fib(dueDays: number, success: boolean): number {
        let prev = 0;
        let curr = 1;
        while (curr <= dueDays) {
            const temp = curr;
            curr = prev + curr;
            prev = temp;
        }
        const newDueDays = success ? curr : curr - prev;
        return newDueDays;
    }
}

export default new SRS();

interface ITab {
    options: string[];
    selected: string;
    setSelected: (selected: string) => void;
}

export default function Tab(props: ITab) {
    const { options, selected, setSelected } = props;
    return (
        <div className="tab">
            {options.map((option) => (
                <div
                    key={option}
                    className={option === selected ? "tab-item tab-item-active" : "tab-item"}
                    onClick={() => setSelected(option)}
                >
                    {option}
                </div>
            ))}
        </div>
    );
}

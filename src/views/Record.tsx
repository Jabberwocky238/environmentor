const dataaa = [
    { ty: "1", name: "", age: "",}
]

export default function Record(props: {style?: React.CSSProperties}) {
    const { style } = props;

    return (
        <div style={style} className="row">
            <h1>Record</h1>
        </div>
    );
}
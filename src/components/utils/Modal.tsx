import ReactDOM from "react-dom";

interface ModalProps {
    title?: string;
    isOpen: boolean;
    onClose: () => void;
    children: React.ReactNode;
}

export default function Modal(props: ModalProps) {
    const { title, isOpen, onClose, children } = props;

    if (!isOpen) return null;

    return ReactDOM.createPortal(
        <div className="modal">
            <div className="modal-overlay" onClick={onClose} />
            <div className="modal-content">
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: 'center', marginBottom: "10px" }}>
                    <strong className='modal-title'>{title}</strong>
                    <button onClick={onClose}>
                        &times; 
                    </button>
                </div>
                {children}
            </div>
        </div>,
        document.body
    );
};

import styles from "./ConnectDialog.module.css";

interface DeepLinkErrorDialogProps {
  message: string;
  onClose: () => void;
}

export default function DeepLinkErrorDialog({ message, onClose }: DeepLinkErrorDialogProps) {
  return (
    <div
      className={styles.overlay}
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div className={styles.box}>
        <h3 className={styles.title}>Cannot connect</h3>
        <p className={styles.status}>{message}</p>
        <button onClick={onClose} className={styles.cancelBtn}>
          CLOSE
        </button>
      </div>
    </div>
  );
}

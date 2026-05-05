import { createContext, useContext, useState, type ReactNode } from 'react'

type ToastType = 'success' | 'error' | 'info'
type Toast = { id: number; message: string; type: ToastType }

const ToastContext = createContext<(msg: string, type?: ToastType) => void>(() => {})

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([])

  const notify = (message: string, type: ToastType = 'info') => {
    const id = Date.now()
    setToasts(prev => [...prev, { id, message, type }])
    setTimeout(() => setToasts(prev => prev.filter(t => t.id !== id)), 4000)
  }

  return (
    <ToastContext.Provider value={notify}>
      {children}
      <div className="toast toast-end toast-bottom z-50">
        {toasts.map(t => (
          <div
            key={t.id}
            className={`alert shadow-lg ${
              t.type === 'success' ? 'alert-success' :
              t.type === 'error' ? 'alert-error' :
              'alert-info'
            }`}
          >
            <span>{t.message}</span>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  )
}

export const useToast = () => useContext(ToastContext)

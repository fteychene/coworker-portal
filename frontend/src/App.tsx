import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { isAuthenticated } from './auth'
import { ProtectedRoute } from './components/ProtectedRoute'
import { ToastProvider } from './lib/toast'
import { CreateBill } from './pages/CreateBill'
import { Dashboard } from './pages/Dashboard'
import { GuestBuy } from './pages/GuestBuy'
import { GuestSummary } from './pages/GuestSummary'
import { Landing } from './pages/Landing'
import { Profile } from './pages/Profile'

function App() {
  return (
    <ToastProvider>
    <BrowserRouter>
      <Routes>
        <Route
          path="/login"
          element={isAuthenticated() ? <Navigate to="/dashboard" replace /> : <Landing />}
        />
        <Route
          path="/dashboard"
          element={<ProtectedRoute><Dashboard /></ProtectedRoute>}
        />
        <Route
          path="/bills/new"
          element={<ProtectedRoute><CreateBill /></ProtectedRoute>}
        />
        <Route path="/buy" element={<GuestBuy />} />
        <Route path="/buy/summary/:token" element={<GuestSummary />} />
        <Route path="/profile" element={<ProtectedRoute><Profile /></ProtectedRoute>} />
        <Route path="*" element={<Navigate to="/dashboard" replace />} />
      </Routes>
    </BrowserRouter>
    </ToastProvider>
  )
}

export default App

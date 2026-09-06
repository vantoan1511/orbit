import test from 'node:test'
import assert from 'node:assert/strict'
import {
  isValidCron,
  isValidHost,
  isValidK8sLabel,
  isValidK8sName,
  isValidPath,
  isValidPort,
  parseRuleSummary,
  sanitizeK8sLabel
} from '../validators.ts'

test('isValidK8sName', () => {
  assert.equal(isValidK8sName('my-app'), true)
  assert.equal(isValidK8sName('redis-cluster'), true)
  assert.equal(isValidK8sName('my-app.dev'), true)
  assert.equal(isValidK8sName('123-app'), true)

  assert.equal(isValidK8sName(''), false)
  assert.equal(isValidK8sName('-my-app'), false)
  assert.equal(isValidK8sName('my-app-'), false)
  assert.equal(isValidK8sName('.my-app'), false)
  assert.equal(isValidK8sName('MY_APP'), false)
  assert.equal(isValidK8sName('a'.repeat(254)), false)
})

test('isValidK8sLabel', () => {
  assert.equal(isValidK8sLabel('my-app'), true)
  assert.equal(isValidK8sLabel('container-1'), true)

  // Labels forbid dots and uppercase
  assert.equal(isValidK8sLabel('my-app.dev'), false)
  assert.equal(isValidK8sLabel('-container'), false)
  assert.equal(isValidK8sLabel('container-'), false)
  assert.equal(isValidK8sLabel('a'.repeat(64)), false)
})

test('isValidPort', () => {
  assert.equal(isValidPort(80), true)
  assert.equal(isValidPort('8080'), true)
  assert.equal(isValidPort(1), true)
  assert.equal(isValidPort(65535), true)

  assert.equal(isValidPort(0), false)
  assert.equal(isValidPort(-1), false)
  assert.equal(isValidPort(65536), false)
  assert.equal(isValidPort('abc'), false)
  assert.equal(isValidPort(80.5), false)
})

test('isValidHost', () => {
  assert.equal(isValidHost('example.com'), true)
  assert.equal(isValidHost('api.example.com'), true)
  assert.equal(isValidHost('*.example.com'), true)

  assert.equal(isValidHost(''), false)
  assert.equal(isValidHost('http://example.com'), false)
})

test('isValidPath', () => {
  assert.equal(isValidPath('/api'), true)
  assert.equal(isValidPath('/'), true)
  assert.equal(isValidPath(''), true)
  assert.equal(isValidPath('api'), false)
})

test('parseRuleSummary', () => {
  assert.deepEqual(parseRuleSummary('app.example.com -> /api (svc:80)'), {
    host: 'app.example.com',
    path: '/api'
  })
  assert.deepEqual(parseRuleSummary('* -> /'), {
    host: '',
    path: '/'
  })
  assert.equal(parseRuleSummary(''), null)
})

test('isValidCron - standard 5-part expressions', () => {
  assert.equal(isValidCron('* * * * *'), true)
  assert.equal(isValidCron('*/5 * * * *'), true)
  assert.equal(isValidCron('0 0 * * *'), true)
  assert.equal(isValidCron('30 4 1,15 * *'), true)
  assert.equal(isValidCron('0 22 * * 1-5'), true)
  assert.equal(isValidCron('  */10   *   *   *   *  '), true)
})

test('isValidCron - weekday and month names', () => {
  assert.equal(isValidCron('0 9 * * MON-FRI'), true)
  assert.equal(isValidCron('0 0 1 JAN *'), true)
  assert.equal(isValidCron('0 0 * * SUN,SAT'), true)
})

test('isValidCron - macros', () => {
  assert.equal(isValidCron('@yearly'), true)
  assert.equal(isValidCron('@annually'), true)
  assert.equal(isValidCron('@monthly'), true)
  assert.equal(isValidCron('@weekly'), true)
  assert.equal(isValidCron('@daily'), true)
  assert.equal(isValidCron('@midnight'), true)
  assert.equal(isValidCron('@hourly'), true)
})

test('isValidCron - invalid expressions', () => {
  assert.equal(isValidCron(''), false)
  assert.equal(isValidCron('   '), false)
  assert.equal(isValidCron('* * * *'), false) // 4 parts
  assert.equal(isValidCron('* * * * * *'), false) // 6 parts
  assert.equal(isValidCron('invalid macro'), false)
  assert.equal(isValidCron('* * * * $'), false)
})

test('sanitizeK8sLabel', () => {
  assert.equal(sanitizeK8sLabel('my-app'), 'my-app')
  assert.equal(sanitizeK8sLabel('redis.cluster'), 'redis-cluster')
  assert.equal(sanitizeK8sLabel('My-App.dev.service'), 'my-app-dev-service')
  assert.equal(sanitizeK8sLabel('-bad-name-'), 'bad-name')
  assert.equal(sanitizeK8sLabel(''), 'main')
  assert.equal(sanitizeK8sLabel('', 'default-container'), 'default-container')

  // Truncation to 63 chars
  const longName = 'a'.repeat(80)
  assert.equal(sanitizeK8sLabel(longName).length, 63)
  assert.equal(sanitizeK8sLabel(longName), 'a'.repeat(63))
})
